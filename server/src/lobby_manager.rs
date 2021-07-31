use crate::util::WsReceiver;
use std::fmt::Debug;
use super::{
    util::{
        WsSender
    }
};
use rand::Rng;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub enum LobbyManagerEvent {
    CreateNewLobby {
        ws_read: WsReceiver, 
        ws_write: WsSender 
    },
    JoinLobby {
        ws_read: WsReceiver,
        ws_write: WsSender,
        lobby_id: u64
    },
}

impl Debug for LobbyManagerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateNewLobby { ws_read: _,ws_write:  _} => {
                write!(f, "LobbyEvent::CreateNewLobby")
            },
            Self::JoinLobby { ws_read: _, ws_write: _, lobby_id } => {
                write!(f, "LobbyEvent::CreateNewLobby{{ lobby_id: {} }}", lobby_id)
            }
        }
    }
}


pub struct LobbyManager {
    lobbies: HashMap<usize, ()>,
}

impl LobbyManager {
    pub fn new() -> Self {
        LobbyManager {
            lobbies: HashMap::new()
        }
    }

    /*
    Runs the lobby manager. This blocks and processes events until the sending part of the channel is dropped
    */
    pub async fn run(&mut self, mut receiver: mpsc::Receiver<LobbyManagerEvent>, sender: mpsc::Sender<LobbyManagerEvent>) {
        while let Some(event) = receiver.recv().await {
            match event {
                LobbyManagerEvent::JoinLobby { ws_read, ws_write, lobby_id } => {
                    //TODO
                }
                LobbyManagerEvent::CreateNewLobby { ws_read, ws_write} => {
                    let new_id = self.generate_lobby_id();
                    //TODO
                }
            }
        }
    }

    fn generate_lobby_id(&self) -> usize {
        loop {
            let id: usize = rand::thread_rng().gen();
            if !self.lobbies.contains_key(&id) {
                return id;
            }
        }
    }
}
