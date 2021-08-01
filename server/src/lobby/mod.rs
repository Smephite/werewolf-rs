mod client_manager;

use super::{
    lobby_manager::LobbyManagerEvent,
    util::{WsReceiver, WsSender},
};
use std::fmt::Debug;
use tokio::sync::mpsc;

pub enum GameLobbyEvent {
    NewConnection {
        ws_read: WsReceiver,
        ws_write: WsSender,
    },
}



pub struct GameLobby {
    id: u64,
    lobby_manager_sender: mpsc::Sender<LobbyManagerEvent>,
    receiver: mpsc::Receiver<GameLobbyEvent>,
    sender: mpsc::Sender<GameLobbyEvent>,
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
            },
            sender,
        )
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.receiver.recv().await {}
    }
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