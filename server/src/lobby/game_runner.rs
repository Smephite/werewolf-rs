use std::{cmp::Ordering, collections::HashMap, mem};

use anyhow::Error;
use futures::{stream::FuturesUnordered, Future, StreamExt};
use rand::{prelude::SliceRandom, Rng};
use tokio::{
    select,
    sync::{broadcast, mpsc, oneshot},
};
use werewolf_rs::{
    game::{CauseOfDeath, Role, RoleData},
    packet::{InteractionFollowup, InteractionRequest, InteractionResponse},
    util::{InteractionId, PlayerId},
};

use super::{
    client_manager::ClientEvent, roles::ServerRole, GameConfig, GameLobby, GameLobbyEvent, Player,
};

/*
A struct that handles the game logic.
This does not contain the game state, as that is handled by the GameLobby
*/
pub struct GameRunner {
    game_config: GameConfig,
    lobby_sender: mpsc::Sender<GameLobbyEvent>,
    game_cancel: broadcast::Sender<()>, //This is mainly intented to create new receivers for the channel
}

impl GameRunner {
    /*
    Creates a new game runner and a broadcast sender that can be used to cancel the game (by stopping all created tasks at their next .await)
    */
    pub fn new(
        game_config: GameConfig,
        lobby_sender: mpsc::Sender<GameLobbyEvent>,
        game_cancel: broadcast::Sender<()>,
    ) -> Self {
        GameRunner {
            game_config,
            lobby_sender,
            game_cancel,
        }
    }

    pub async fn start(mut self) {
        tokio::spawn(async move {
            let mut game_cancel = self.game_cancel.subscribe();
            select! {
                biased;
                _ = game_cancel.recv() => {}
                res = self.run() => {
                    if let Err(e) = res {
                        error!("Error running game: {:?}", e);
                    }
                }
            };
        });
    }

    async fn run(&mut self) -> Result<(), Error> {
        self.assign_roles().await?;

        //The main game loop
        loop {
            self.run_night().await?;
            self.run_day().await?;
        }
    }

    async fn assign_roles(&mut self) -> Result<(), Error> {
        let mut client_ids: Vec<PlayerId> =
            GameLobby::access_game_data(&self.lobby_sender, |game_data, _| {
                game_data.players.keys().copied().collect()
            })
            .await?;
        client_ids.shuffle(&mut rand::thread_rng());
        let mut client_roles: Vec<RoleData> = Vec::with_capacity(client_ids.len());
        let mut remaining_roles = self.game_config.roles.clone();
        for _ in client_ids.iter() {
            if remaining_roles.is_empty() {
                //If there are not enough roles set, assign the rest of the clients as villagers
                client_roles.push(RoleData::new(&Role::Villager));
            } else {
                //Sample a random remaining role
                let idx = rand::thread_rng().gen_range(0..remaining_roles.len());
                client_roles.push(RoleData::new(&remaining_roles[idx]));
                remaining_roles.swap_remove(idx);
            }
        }
        GameLobby::access_game_data(&self.lobby_sender, move |game_data, _| {
            for (client_id, assigned_role) in client_ids.iter().zip(client_roles) {
                match game_data.players.get_mut(client_id) {
                    None => {
                        warn!("A client seems to have disconnected during role assignment");
                    }
                    Some(player) => {
                        player.role_data = assigned_role;
                    }
                }
            }
        })
        .await?;
        self.lobby_sender.send(GameLobbyEvent::SendUpdate).await?;
        Ok(())
    }

    async fn run_night(&mut self) -> Result<(), Error> {
        //A list of all the involved roles and whether they have already been run in this night
        let unique_roles = GameLobby::access_game_data(&self.lobby_sender, |game_data, _| {
            let mut unique_roles: Vec<(Role, bool)> = Vec::new();
            for player in game_data.players.values().filter(|p| p.is_alive) {
                if !unique_roles.contains(&(player.role_data.get_role(), false)) {
                    unique_roles.push((player.role_data.get_role(), false));
                }
            }
            unique_roles
        })
        .await?;
        //Send a message on this channel whenever a role has finished running
        let (role_finish_send, mut role_finish_rec) = mpsc::channel::<()>(1);
        role_finish_send.send(()).await?;
        while let Some(()) = role_finish_rec.recv().await {
            let mut finished = true;
            let unfinished_roles = unique_roles
                .iter()
                .filter(|(_, has_run)| !has_run)
                .map(|(role, _)| role);
            for role in unfinished_roles.clone() {
                finished = false;
                //Check whether all dependencies have been fulfilled
                //This could be done a lot more efficiently but it's unlikely to ever be performance-critical
                let mut deps_fulfilled = true;
                'dep_check: for unfinished_role in unfinished_roles.clone() {
                    for dependency in role.dependencies_in_night() {
                        if dependency == *unfinished_role {
                            deps_fulfilled = false;
                            break 'dep_check;
                        }
                    }
                }

                if deps_fulfilled {
                    let finish_send = role_finish_send.clone();
                    let role = role.clone();
                    let lobby_sender = self.lobby_sender.clone();
                    self.spawn_task(async move {
                        role.run_night_turn(lobby_sender).await?;
                        finish_send.send(()).await?;
                        Ok(())
                    })
                    .await;
                }
            }
            if finished {
                break;
            }
        }

        //Apply the changes that happened during the night (but only take effect now)
        self.lobby_sender.send(GameLobbyEvent::ApplyDeaths).await?;
        Ok(())
    }

    async fn run_day(&mut self) -> Result<(), Error> {
        //TODO Add some more information to the nomination_vote function to indicate what is being voted on
        let village_vote = Self::nomination_vote(&self.lobby_sender).await?;
        let mut voted_for: HashMap<PlayerId, u32> = HashMap::new();
        for (_, vote) in village_vote {
            voted_for
                .entry(vote)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        /*
        TODO Add different options for how to handle ambiguous voting results, such as:
        * Rerun the vote with only the most voted for players as an option
        * Kill no one this round
        * Kill all players that have been selected
        */
        let mut max_found_votes = 0;
        let mut vote_unambiguous = true;
        let mut voted_player = None;
        for (player, &count) in voted_for.iter() {
            match count.cmp(&max_found_votes) {
                Ordering::Greater => {
                    max_found_votes = count;
                    vote_unambiguous = true;
                    voted_player = Some(player);
                }
                Ordering::Equal => {
                    vote_unambiguous = false;
                }
                Ordering::Less => {}
            }
        }
        if let Some(voted_player) = voted_player {
            if vote_unambiguous {
                self.lobby_sender
                    .send(GameLobbyEvent::KillPlayer(
                        *voted_player,
                        CauseOfDeath::VillageVote,
                    ))
                    .await?;
            }
        }
        todo!();
    }

    //Spawn a new task that stops when the game is cancelled
    async fn spawn_task<T>(&mut self, task: T)
    where
        T: Future<Output = Result<(), Error>> + Send + 'static,
    {
        let mut game_cancel = self.game_cancel.subscribe();
        tokio::spawn(async move {
            select! {
                biased;
                _ = game_cancel.recv() => {}
                res = task => {
                    if let Err(e) = res {
                        error!("Error running game subtask: {:?}", e);
                    }
                }

            }
        });
    }

    /*
    Runs a nomination vote where all alive players can vote and be nominated
    Returns the result as a vector of (player, vote) tuples
    */
    async fn nomination_vote(
        lobby_sender: &mpsc::Sender<GameLobbyEvent>,
    ) -> Result<Vec<(PlayerId, PlayerId)>, Error> {
        enum VotingStatus {
            NotVoting,
            NominationPending,
            NominationFinished(Option<PlayerId>),
            VoteFinished(PlayerId),
        }

        //Mapping from client id to (client_sender, voting_status)
        let mut clients: HashMap<PlayerId, (mpsc::Sender<ClientEvent>, VotingStatus)> =
            GameLobby::access_game_data(lobby_sender, |game_data, clients| {
                let mut ret_clients = HashMap::new();
                for (player_id, player) in game_data.players.iter() {
                    ret_clients.insert(
                        *player_id,
                        (
                            clients.get(player_id).unwrap().clone(),
                            if player.is_alive {
                                VotingStatus::NominationPending
                            } else {
                                VotingStatus::NotVoting
                            },
                        ),
                    );
                }
                ret_clients
            })
            .await?;

        //Create the interactions and collect the interaction ID for each client in a hashmap
        let mut id_futs = FuturesUnordered::new();
        let (interaction_send, mut interaction_receive) = mpsc::channel(8);
        for (&id, (sender, voting_status)) in clients.iter() {
            let (id_send, id_receive) = oneshot::channel();
            sender
                .send(ClientEvent::CreateInteraction(
                    InteractionRequest::NvBegin {
                        nominatable_player_ids: clients
                            .iter()
                            .filter(|(_, (_, voting_status))| {
                                !matches!(voting_status, VotingStatus::NotVoting)
                            })
                            .map(|(id, _)| *id)
                            .collect(),
                        can_vote: !matches!(voting_status, &VotingStatus::NotVoting),
                    },
                    interaction_send.clone(),
                    id_send,
                ))
                .await?;
            id_futs.push(async move { (id, id_receive.await) });
        }
        let mut interaction_ids: HashMap<PlayerId, InteractionId> = HashMap::new();
        while let Some((user_id, interaction_id)) = id_futs.next().await {
            let interaction_id = interaction_id?;
            interaction_ids.insert(user_id, interaction_id);
        }

        //Accept all nominations
        while let Some((client_id, response)) = interaction_receive.recv().await {
            match response {
                InteractionResponse::NvNominate { nominated_player } => {
                    let (_, voting_status) = clients.get_mut(&client_id).unwrap();
                    if let VotingStatus::NominationPending = voting_status {
                        *voting_status = VotingStatus::NominationFinished(nominated_player);
                        //Notify all other clients of the nomination
                        for (other_client, (sender, _)) in clients.iter() {
                            let interaction_id = interaction_ids[other_client];
                            sender
                                .send(ClientEvent::FollowupInteraction(
                                    interaction_id,
                                    InteractionFollowup::NvNewNomination {
                                        nominated_player,
                                        nominated_by: client_id,
                                    },
                                ))
                                .await?;
                        }

                        //Break out of the loop once all pending nominations have been received
                        if clients.values().all(|(_, voting_status)| {
                            !matches!(voting_status, VotingStatus::NominationPending)
                        }) {
                            break;
                        }
                    } else {
                        warn!("Received Nomination from client that is not currently allowed to nominate");
                    }
                }
                r => {
                    warn!(
                        "Received invalid interaction response during nomination phase: {:?}",
                        r
                    );
                }
            }
        }
        for (client_id, (sender, _)) in clients.iter() {
            let interaction_id = interaction_ids[client_id];
            sender
                .send(ClientEvent::FollowupInteraction(
                    interaction_id,
                    InteractionFollowup::NvNominationsFinished,
                ))
                .await?;
        }

        //Accept votes
        while let Some((client_id, response)) = interaction_receive.recv().await {
            match response {
                InteractionResponse::NvVote { player_id } => {
                    let (_, voting_status) = clients.get_mut(&client_id).unwrap();
                    if let VotingStatus::NominationFinished(_) = voting_status {
                        *voting_status = VotingStatus::VoteFinished(player_id);

                        //Break out of the loop once all pending votes have been received
                        if clients.values().all(|(_, voting_status)| {
                            !matches!(voting_status, VotingStatus::NominationFinished(_))
                        }) {
                            break;
                        }
                    } else {
                        warn!("Received vote from client that is not currently allowed to vote");
                    }
                }
                r => {
                    warn!(
                        "Received invalid interaction response during voting phase: {:?}",
                        r
                    );
                }
            }
        }

        //Send the result to all clients and return it
        let vote_result: Vec<(PlayerId, PlayerId)> = clients
            .iter()
            .filter_map(|(client_id, (_, voting_status))| match voting_status {
                VotingStatus::VoteFinished(vote) => Some((*client_id, *vote)),
                _ => None,
            })
            .collect();
        for (client, (sender, _)) in clients.iter() {
            let interaction_id = interaction_ids[client];
            sender
                .send(ClientEvent::FollowupInteraction(
                    interaction_id,
                    InteractionFollowup::NvVoteFinished {
                        votes: vote_result.clone(),
                    },
                ))
                .await?;
            sender
                .send(ClientEvent::CloseInteraction(interaction_id))
                .await?;
        }
        Ok(vote_result)
    }

    /*
    Runs a unanimous vote with the players who are participating and those who are selectable defined by the given functions
    */
    async fn unanimous_vote<F>(
        lobby_sender: &mpsc::Sender<GameLobbyEvent>,
        mut participating: F,
        mut selectable: F,
    ) -> Result<(), Error>
    where
        F: FnMut(&Player) -> bool + Send + Sync + 'static,
    {
        enum VotingStatus {
            NotParticipating,
            NoVote,
            VotingFor(PlayerId),
            ConfirmedVote(PlayerId),
        }
        let (clients, selectable) =
            GameLobby::access_game_data(lobby_sender, move |game_data, clients| {
                let mut ret_clients: HashMap<PlayerId, (mpsc::Sender<ClientEvent>, VotingStatus)> =
                    HashMap::new();
                let mut ret_selectable: Vec<PlayerId> = Vec::new();
                for (id, player) in game_data.players.iter() {
                    if participating(player) {
                        ret_clients.insert(
                            *id,
                            (clients.get(id).unwrap().clone(), VotingStatus::NoVote),
                        );
                    } else if player.role_data.get_role() == Role::Spectator {
                        ret_clients.insert(
                            *id,
                            (
                                clients.get(id).unwrap().clone(),
                                VotingStatus::NotParticipating,
                            ),
                        );
                    }
                    if selectable(player) {
                        ret_selectable.push(*id);
                    }
                }
                (ret_clients, ret_selectable)
            })
            .await?;
        todo!();
        Ok(())
    }
}
