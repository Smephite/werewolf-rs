mod client_manager;
mod game_runner;
mod roles;

use crate::{lobby::game_runner::GameRunner, util::generate_random_id};

use super::{
    lobby_manager::LobbyManagerEvent,
    util::{WsReceiver, WsSender},
};
use anyhow::Error;
use client_manager::{ClientEvent, ClientManager};
use std::{collections::HashMap, fmt::Debug};
use tokio::sync::{broadcast, mpsc, oneshot};

pub enum GameLobbyEvent {
    NewConnection {
        ws_read: WsReceiver,
        ws_write: WsSender,
    },
    ConnectionLost {
        client_id: u64,
    },
    StartGame {
        client_id: u64,
    },
    //Run an arbitrary (non-blocking) function on the game data
    AccessGameData(Box<dyn FnOnce(&mut GameData) + Send + Sync>),
}

struct Client {
    sender: mpsc::Sender<ClientEvent>,
    is_lobby_host: bool,
}

pub struct GameData {}

pub struct GameLobby {
    id: u64,
    lobby_manager_sender: mpsc::Sender<LobbyManagerEvent>,
    receiver: mpsc::Receiver<GameLobbyEvent>,
    sender: mpsc::Sender<GameLobbyEvent>,
    game_cancel: broadcast::Sender<()>,

    clients: HashMap<u64, Client>,
    game_data: Option<GameData>,
}

impl GameLobby {
    pub fn new(
        id: u64,
        lobby_manager_sender: mpsc::Sender<LobbyManagerEvent>,
    ) -> (Self, mpsc::Sender<GameLobbyEvent>) {
        //The channel to send events to this lobby
        let (sender, receiver) = mpsc::channel(8);
        //The channel to cancel a running game
        let (cancel_sender, cancel_receiver) = broadcast::channel(1);
        (
            GameLobby {
                id,
                lobby_manager_sender,
                receiver,
                sender: sender.clone(),
                game_cancel: cancel_sender,

                clients: HashMap::new(),
                game_data: None,
            },
            sender,
        )
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.receiver.recv().await {
            match event {
                GameLobbyEvent::NewConnection { ws_read, ws_write } => {
                    let client_id = generate_random_id(&self.clients);
                    let (client_manager, client_sender) = ClientManager::new(
                        self.id,
                        client_id,
                        ws_write,
                        ws_read,
                        self.sender.clone(),
                    )
                    .await;
                    client_manager.start().await;
                    let client = Client {
                        sender: client_sender,
                        is_lobby_host: self.clients.values().all(|c| !c.is_lobby_host),
                    };
                    self.clients.insert(client_id, client);
                }
                GameLobbyEvent::ConnectionLost { client_id } => {
                    //TODO Consider adding a "reconnect" feature if connection loss becomes a problem
                    self.clients.remove(&client_id);
                }
                GameLobbyEvent::StartGame { client_id } => {
                    let client = self.clients.get(&client_id).unwrap();
                    if client.is_lobby_host {
                        let game_runner =
                            GameRunner::new(self.sender.clone(), self.game_cancel.subscribe());
                        game_runner.start().await;
                    } else {
                        warn!("Received start game request by client without permission");
                    }
                }
                GameLobbyEvent::AccessGameData(f) => match self.game_data.as_mut() {
                    None => {
                        error!("Tried to access non-existing game data");
                    }
                    Some(data) => {
                        f(data);
                    }
                },
            }
        }
    }

    /*
    Executes an arbitrary function on the game data and returns the result.
    The function should complete fast, as otherwise it will stall the whole lobby
    */
    pub async fn access_game_data<F, R>(
        sender: mpsc::Sender<GameLobbyEvent>,
        f: F,
    ) -> Result<R, Error>
    where
        F: FnOnce(&mut GameData) -> R + Send + Sync + 'static,
        R: Send + 'static,
    {
        let (callback_send, callback_rec) = oneshot::channel::<R>();
        let f_callback = move |game_data: &mut GameData| {
            let result = f(game_data);
            callback_send.send(result).ok();
        };
        sender
            .send(GameLobbyEvent::AccessGameData(Box::new(f_callback)))
            .await?;
        Ok(callback_rec.await?)
    }
}

impl Debug for GameLobbyEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameLobbyEvent")
    }
}
