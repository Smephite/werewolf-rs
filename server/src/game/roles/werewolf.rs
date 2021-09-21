use std::collections::HashMap;

use crate::game::{client_manager::ClientEvent, GameLobby, Player};

use super::ServerRole;
use anyhow::Error;
use async_trait::async_trait;
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::{mpsc, oneshot};
use werewolf_rs::{
    game::{CauseOfDeath, Role},
    packet::{InteractionFollowup, InteractionRequest, InteractionResponse},
    util::{InteractionId, PlayerId},
};

pub struct Werewolf;

#[async_trait]
impl ServerRole for Werewolf {
    async fn run_night_turn(
        &self,
        lobby_sender: &tokio::sync::mpsc::Sender<crate::game::GameLobbyEvent>,
    ) -> Result<(), anyhow::Error> {
        enum VotingStatus {
            NotParticipating,
            NoVote,
            VotingFor(PlayerId),
            LockedVote(PlayerId),
        }
        type ClientMap = HashMap<PlayerId, (mpsc::Sender<ClientEvent>, VotingStatus)>;

        fn participating(player: &Player) -> bool {
            player.role_data.get_role() == Role::Werewolf
        }
        fn selectable(player: &Player) -> bool {
            player.is_alive
        }

        //Get a list of participating and selectable players
        let (mut clients, selectable): (ClientMap, Vec<PlayerId>) =
            GameLobby::access_game_data(lobby_sender, move |game_data, clients| {
                let mut ret_clients: ClientMap = HashMap::new();
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

        let (interaction_send, mut interaction_receive) = mpsc::channel(8);
        //Create the interactions
        let mut id_futs = FuturesUnordered::new();
        for (id, (sender, status)) in clients.iter() {
            let (id_send, id_receive) = oneshot::channel();
            sender
                .send(ClientEvent::CreateInteraction(
                    InteractionRequest::WvBegin {
                        selectable_players: selectable.clone(),
                        can_vote: matches!(status, VotingStatus::NoVote),
                    },
                    interaction_send.clone(),
                    id_send,
                ))
                .await?;
            id_futs.push(async move { (id, id_receive.await) })
        }
        //Receive the interaction ids
        let mut interaction_ids: HashMap<PlayerId, InteractionId> = HashMap::new();
        while let Some((player_id, interaction_id)) = id_futs.next().await {
            interaction_ids.insert(*player_id, interaction_id?);
        }
        drop(id_futs);

        async fn send_update(
            update: &InteractionFollowup,
            clients: &ClientMap,
            interaction_ids: &HashMap<PlayerId, InteractionId>,
        ) -> Result<(), Error> {
            for (id, (sender, _)) in clients.iter() {
                let interaction_id = interaction_ids.get(id).unwrap();
                sender
                    .send(ClientEvent::FollowupInteraction(
                        *interaction_id,
                        (*update).clone(),
                    ))
                    .await?;
            }
            Ok(())
        }

        let mut final_vote: Option<PlayerId> = None;
        //Main voting event loop
        while let Some((player_id, response)) = interaction_receive.recv().await {
            match response {
                InteractionResponse::WvVote { vote } => {
                    let (_, status) = clients.get_mut(&player_id).unwrap();
                    match status {
                        VotingStatus::NoVote | VotingStatus::VotingFor(_) => {
                            *status = VotingStatus::VotingFor(vote);
                            send_update(
                                &InteractionFollowup::WvNewVote {
                                    vote,
                                    voted_by: player_id,
                                },
                                &clients,
                                &interaction_ids,
                            )
                            .await?;
                        }
                        _ => {
                            warn!("Received illegal vote during werewolf vote");
                        }
                    }
                }
                InteractionResponse::WvLockVote => {
                    //Locking the vote is only allowed if all werewolves vote for the same player
                    let mut lock_allowed = true;
                    let mut voting_statuses = clients.values().map(|(_, status)| status);
                    if let VotingStatus::VotingFor(first_vote)
                    | VotingStatus::LockedVote(first_vote) = *voting_statuses.next().unwrap()
                    {
                        for status in voting_statuses {
                            if let VotingStatus::VotingFor(vote) | VotingStatus::LockedVote(vote) =
                                *status
                            {
                                if vote != first_vote {
                                    lock_allowed = false;
                                    break;
                                }
                            } else {
                                lock_allowed = false;
                                break;
                            }
                        }
                    } else {
                        lock_allowed = false;
                    }
                    if !lock_allowed {
                        warn!("Received WvLockVote packet while not all werewolves have the same vote");
                    }

                    let (_, status) = clients.get_mut(&player_id).unwrap();
                    if let VotingStatus::VotingFor(vote) = status {
                        let vote = *vote;
                        *status = VotingStatus::LockedVote(vote);
                        send_update(
                            &InteractionFollowup::WvLockedVote {
                                vote,
                                voted_by: player_id,
                            },
                            &clients,
                            &interaction_ids,
                        )
                        .await?;
                        if clients
                            .values()
                            .all(|(_, status)| matches!(status, VotingStatus::LockedVote(_)))
                        {
                            //The vote ends when all werewolves locked their vote
                            final_vote = Some(vote);
                            break;
                        }
                    } else {
                        warn!("Received WvLockVote packet by a player who doesn't have an (unlocked) vote");
                    }
                }
                r => {
                    warn!("Received invalid response during werewolf vote: {:?}", r);
                }
            }
        }

        //Broadcast and apply the vote result
        send_update(
            &InteractionFollowup::WvVoteFinished { vote: final_vote },
            &clients,
            &interaction_ids,
        )
        .await?;
        if let Some(vote) = final_vote {
            GameLobby::access_game_data(lobby_sender, move |game_data, _| {
                game_data
                    .dying_players
                    .push((vote, CauseOfDeath::Werewolves));
            })
            .await?;
        }

        Ok(())
    }
}
