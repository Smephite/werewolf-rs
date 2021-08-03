mod client_manager;

use crate::util::generate_random_id;

use super::{
    lobby_manager::LobbyManagerEvent,
    util::{WsReceiver, WsSender},
};
use client_manager::{ClientEvent, ClientManager};
use std::{collections::HashMap, fmt::Debug};
use tokio::sync::mpsc;

pub enum GameLobbyEvent {
    NewConnection {
        ws_read: WsReceiver,
        ws_write: WsSender,
    },
    ConnectionLost {
        client_id: u64,
    },
    StartGame {
        client_id: u64
    }
}

struct Client {
    sender: mpsc::Sender<ClientEvent>,
    is_lobby_host: bool
}

pub struct GameLobby {
    id: u64,
    lobby_manager_sender: mpsc::Sender<LobbyManagerEvent>,
    receiver: mpsc::Receiver<GameLobbyEvent>,
    sender: mpsc::Sender<GameLobbyEvent>,

    clients: HashMap<u64, Client>,
}

impl GameLobby {
    pub fn new(
        id: u64,
        lobby_manager_sender: mpsc::Sender<LobbyManagerEvent>,
    ) -> (Self, mpsc::Sender<GameLobbyEvent>) {
        let (sender, receiver) = mpsc::channel(8);
        (
            GameLobby {
                id,
                lobby_manager_sender,
                receiver,
                sender: sender.clone(),
                clients: HashMap::new(),
            },
            sender,
        )
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.receiver.recv().await {
            match event {
                GameLobbyEvent::NewConnection { ws_read, ws_write } => {
                    let client_id = generate_random_id(&self.clients);
                    let (mut client_manager, client_sender) = ClientManager::new(
                        self.id,
                        client_id,
                        ws_write,
                        ws_read,
                        self.sender.clone(),
                    )
                    .await;
                    tokio::spawn(async move {
                        client_manager.run().await;
                    });
                    let client = Client {
                        sender: client_sender,
                        is_lobby_host: self.clients.values().all(|c| !c.is_lobby_host)
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
                        todo!();
                    } else {
                        warn!("Received start game request by client without permission");
                    }
                }
            }
        }
    }
}

impl Debug for GameLobbyEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameLobbyEvent")
    }
}
