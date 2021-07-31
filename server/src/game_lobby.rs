use std::fmt::Debug;

use super::{
    lobby_manager::LobbyManagerEvent,
    util::{WsReceiver, WsSender},
};
use tokio::sync::mpsc;

pub enum GameLobbyEvent {
    NewConnection {
        ws_read: WsReceiver,
        ws_write: WsSender,
    },
}

impl Debug for GameLobbyEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewConnection {
                ws_read: _,
                ws_write: _,
            } => {
                write!(f, "GameLobbyEvent::NewConnection")
            }
        }
    }
}

pub struct GameLobby {
    id: u64,
    lobby_manager_sender: mpsc::Sender<LobbyManagerEvent>,
    receiver: mpsc::Receiver<GameLobbyEvent>,
    sender: mpsc::Sender<GameLobbyEvent>,
}

impl GameLobby {
    pub fn new(id: u64, lobby_manager_sender: mpsc::Sender<LobbyManagerEvent>) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        GameLobby {
            id,
            lobby_manager_sender,
            receiver,
            sender,
        }
    }

    pub fn get_sender(&self) -> mpsc::Sender<GameLobbyEvent> {
        self.sender.clone()
    }

    pub async fn run(&mut self) {}
}
