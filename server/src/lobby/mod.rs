mod client_manager;
mod game_runner;
mod roles;

use crate::lobby::game_runner::GameRunner;

use super::{
    lobby_manager::LobbyManagerEvent,
    util::{WsReceiver, WsSender},
};
use anyhow::Error;
use client_manager::{ClientEvent, ClientManager};
use std::{collections::HashMap, fmt::Debug, mem};
use tokio::sync::{broadcast, mpsc, oneshot};
use werewolf_rs::{
    game::{CauseOfDeath, Role, RoleData},
    packet::PacketToClient,
    util::{Id, LobbyId, PlayerId},
};

type GameDataFunction =
    Box<dyn FnOnce(&mut GameData, &HashMap<PlayerId, mpsc::Sender<ClientEvent>>) + Send + Sync>;

pub enum GameLobbyEvent {
    NewConnection {
        ws_read: WsReceiver,
        ws_write: WsSender,
    },
    ConnectionLost {
        client_id: PlayerId,
    },
    StartGame {
        requested_by: PlayerId,
    },
    KillPlayer(PlayerId, CauseOfDeath),
    //Kill all dying players. This happens at the end of each night
    ApplyDeaths,
    //Send an update to all connected clients with the updated game data
    SendUpdate,
    //Run an arbitrary (non-blocking) function on the game data
    AccessGameData(GameDataFunction),
}

#[derive(Clone)]
pub struct Player {
    role_data: RoleData,
    is_lobby_host: bool,
    is_alive: bool,
}

#[derive(Clone)]
pub struct GameData {
    players: HashMap<PlayerId, Player>,
    dying_players: Vec<(PlayerId, CauseOfDeath)>,
}

#[derive(Clone)]
pub struct GameConfig {
    roles: Vec<Role>,
}

pub struct GameLobby {
    id: LobbyId,
    lobby_manager_sender: mpsc::Sender<LobbyManagerEvent>,
    receiver: mpsc::Receiver<GameLobbyEvent>,
    sender: mpsc::Sender<GameLobbyEvent>,
    game_cancel: broadcast::Sender<()>,

    clients: HashMap<PlayerId, mpsc::Sender<ClientEvent>>,
    game_data: GameData,
    game_config: GameConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig { roles: Vec::new() }
    }
}

impl Default for GameData {
    fn default() -> Self {
        GameData {
            players: HashMap::new(),
            dying_players: Vec::new(),
        }
    }
}

impl GameLobby {
    pub fn new(
        id: LobbyId,
        lobby_manager_sender: mpsc::Sender<LobbyManagerEvent>,
    ) -> (Self, mpsc::Sender<GameLobbyEvent>) {
        //The channel to send events to this lobby
        let (sender, receiver) = mpsc::channel(8);
        //The channel to cancel a running game
        let (cancel_sender, _) = broadcast::channel(1);
        (
            GameLobby {
                id,
                lobby_manager_sender,
                receiver,
                sender: sender.clone(),
                game_cancel: cancel_sender,

                clients: HashMap::new(),
                game_data: GameData::default(),
                game_config: GameConfig::default(),
            },
            sender,
        )
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.receiver.recv().await {
            match event {
                GameLobbyEvent::NewConnection { ws_read, ws_write } => {
                    let client_id = Id::new(&self.clients);
                    let (client_manager, client_sender) = ClientManager::new(
                        self.id,
                        client_id,
                        ws_write,
                        ws_read,
                        self.sender.clone(),
                    )
                    .await;
                    client_manager.start().await;
                    let player = Player {
                        role_data: RoleData::Spectator,
                        is_lobby_host: self.game_data.players.values().all(|c| !c.is_lobby_host),
                        is_alive: false,
                    };
                    self.game_data.players.insert(client_id, player);
                    self.clients.insert(client_id, client_sender);
                    self.send_update().await;
                }
                GameLobbyEvent::ConnectionLost { client_id } => {
                    //TODO Consider adding a "reconnect" feature if connection loss becomes a problem
                    self.clients.remove(&client_id);
                    self.send_update().await;
                }
                GameLobbyEvent::StartGame {
                    requested_by: client_id,
                } => {
                    let player = self.game_data.players.get(&client_id).unwrap();
                    if player.is_lobby_host {
                        let game_runner = GameRunner::new(
                            self.game_config.clone(),
                            self.sender.clone(),
                            self.game_cancel.clone(),
                        );
                        game_runner.start().await;
                    } else {
                        warn!("Received start game request by client without permission");
                    }
                }
                GameLobbyEvent::ApplyDeaths => {
                    for (player, cause) in mem::take(&mut self.game_data.dying_players) {
                        self.kill_player(player, cause).await;
                    }
                }
                GameLobbyEvent::KillPlayer(id, cause) => {
                    self.kill_player(id, cause).await;
                }
                GameLobbyEvent::SendUpdate => {
                    self.send_update().await;
                }
                GameLobbyEvent::AccessGameData(f) => {
                    f(&mut self.game_data, &self.clients);
                }
            }
        }
    }

    /*
    Executes an arbitrary function on the game data and returns the result.
    The function should complete fast, as otherwise it will stall the whole lobby
    */
    pub async fn access_game_data<F, R>(
        sender: &mpsc::Sender<GameLobbyEvent>,
        f: F,
    ) -> Result<R, Error>
    where
        F: FnOnce(&mut GameData, &HashMap<PlayerId, mpsc::Sender<ClientEvent>>) -> R
            + Send
            + Sync
            + 'static,
        R: Send + 'static,
    {
        let (callback_send, callback_rec) = oneshot::channel::<R>();
        let f_callback =
            move |game_data: &mut GameData,
                  clients: &HashMap<PlayerId, mpsc::Sender<ClientEvent>>| {
                let result = f(game_data, clients);
                callback_send.send(result).ok();
            };
        sender
            .send(GameLobbyEvent::AccessGameData(Box::new(f_callback)))
            .await?;
        Ok(callback_rec.await?)
    }

    /*
    Sends the relevant game data to all connected clients
    */
    async fn send_update(&mut self) {
        for sender in self.clients.values() {
            if sender
                .send(ClientEvent::SendUpdate(self.game_data.clone()))
                .await
                .is_err()
            {
                error!("Error sending update to client manager");
            }
        }
    }

    async fn kill_player(&mut self, id: PlayerId, cause: CauseOfDeath) {
        if let Some(player) = self.game_data.players.get_mut(&id) {
            for sender in self.clients.values() {
                let packet =
                    PacketToClient::PlayerDied(id, cause.clone(), player.role_data.get_role());
                if sender.send(ClientEvent::SendPacket(packet)).await.is_err() {
                    error!("Error sending PlayerDied packet to client manager");
                }
            }
            player.is_alive = false;
        } else {
            warn!("Tried to kill non-existing player");
        }
    }
}

impl Debug for GameLobbyEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameLobbyEvent")
    }
}
