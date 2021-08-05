use anyhow::Error;
use futures::Future;
use rand::{prelude::SliceRandom, Rng};
use tokio::{
    select,
    sync::{broadcast, mpsc},
};
use werewolf_rs::game::{Role, RoleData};

use super::{GameConfig, GameLobby, GameLobbyEvent};

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
                _ = game_cancel.recv() => {
                    return;
                }
                res = self.run() => {
                    if let Err(e) = res {
                        error!("Error running game: {:?}", e);
                    }
                }
            };
        });
    }

    async fn run(&mut self) -> Result<(), Error> {
        //Assign the roles
        let mut client_ids: Vec<u64> =
            GameLobby::access_game_data(&self.lobby_sender, |game_data| {
                game_data.players.keys().map(|&id| id).collect()
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
        GameLobby::access_game_data(&self.lobby_sender, move |game_data| {
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

        //The main game loop
        loop {}
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
                _ = game_cancel.recv() => {
                    return;
                }
                res = task => {
                    if let Err(e) = res {
                        error!("Error running game subtask: {:?}", e);
                    }
                }

            }
        });
    }
}
