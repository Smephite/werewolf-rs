use super::{game_lobby::GameLobbyEvent, util::WsSender};
use crate::{game_lobby::GameLobby, util::WsReceiver};
use rand::Rng;
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::sync::mpsc;

pub enum LobbyManagerEvent {
    CreateNewLobby {
        ws_read: WsReceiver,
        ws_write: WsSender,
    },
    JoinLobby {
        ws_read: WsReceiver,
        ws_write: WsSender,
        lobby_id: u64,
    },
}

impl Debug for LobbyManagerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateNewLobby {
                ws_read: _,
                ws_write: _,
            } => {
                write!(f, "LobbyEvent::CreateNewLobby")
            }
            Self::JoinLobby {
                ws_read: _,
                ws_write: _,
                lobby_id,
            } => {
                write!(f, "LobbyEvent::CreateNewLobby{{ lobby_id: {} }}", lobby_id)
            }
        }
    }
}

pub struct LobbyManager {
    lobby_channels: HashMap<u64, mpsc::Sender<GameLobbyEvent>>,
}

impl LobbyManager {
    pub fn new() -> Self {
        LobbyManager {
            lobby_channels: HashMap::new(),
        }
    }

    /*
    Runs the lobby manager. This blocks and processes events until the sending part of the channel is dropped
    */
    pub async fn run(
        &mut self,
        mut receiver: mpsc::Receiver<LobbyManagerEvent>,
        sender: mpsc::Sender<LobbyManagerEvent>,
    ) {
        while let Some(event) = receiver.recv().await {
            match event {
                LobbyManagerEvent::JoinLobby {
                    ws_read,
                    ws_write,
                    lobby_id,
                } => match self.lobby_channels.get(&lobby_id) {
                    Some(lobby_sender) => {
                        if let Err(e) = lobby_sender
                            .send(GameLobbyEvent::NewConnection { ws_read, ws_write })
                            .await
                        {
                            error!("Error sending user to game lobby: {:?}", e);
                        }
                    }
                    None => {}
                },
                LobbyManagerEvent::CreateNewLobby { ws_read, ws_write } => {
                    let new_id = self.generate_lobby_id();
                    let mut lobby = GameLobby::new(new_id, sender.clone());
                    let lobby_sender = lobby.get_sender();
                    self.lobby_channels.insert(new_id, lobby_sender.clone());
                    tokio::spawn(async move {
                        lobby.run().await;
                    });
                    if let Err(e) = lobby_sender
                        .send(GameLobbyEvent::NewConnection { ws_read, ws_write })
                        .await
                    {
                        error!("Error sending user to newly created game lobby: {:?}", e);
                    }
                }
            }
        }
    }

    fn generate_lobby_id(&self) -> u64 {
        loop {
            let id: u64 = rand::thread_rng().gen();
            if !self.lobby_channels.contains_key(&id) {
                return id;
            }
        }
    }
}
