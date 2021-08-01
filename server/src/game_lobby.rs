use std::fmt::Debug;

use crate::util::send_logging;

use super::{
    lobby_manager::LobbyManagerEvent,
    util::{WsReceiver, WsSender},
};
use futures::{SinkExt, StreamExt};
use tokio::{select, sync::mpsc};
use werewolf_rs::packet::{PacketToClient, PacketToServer};

pub enum GameLobbyEvent {
    NewConnection {
        ws_read: WsReceiver,
        ws_write: WsSender,
    },
}

pub enum ClientEvent {}

pub struct GameLobby {
    id: u64,
    lobby_manager_sender: mpsc::Sender<LobbyManagerEvent>,
    receiver: mpsc::Receiver<GameLobbyEvent>,
    sender: mpsc::Sender<GameLobbyEvent>,
}

pub struct ClientManager {
    packet_send: mpsc::Sender<PacketToClient>,
    packet_receive: mpsc::Receiver<PacketToServer>,
    event_receive: mpsc::Receiver<ClientEvent>,
    game_lobby_send: mpsc::Sender<GameLobbyEvent>,
}

impl GameLobby {
    pub fn new(id: u64, lobby_manager_sender: mpsc::Sender<LobbyManagerEvent>) -> (Self, mpsc::Sender<GameLobbyEvent>) {
        let (sender, receiver) = mpsc::channel(8);
        (GameLobby {
            id,
            lobby_manager_sender,
            receiver,
            sender: sender.clone(),
        }, sender)
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.receiver.recv().await {}
    }
}

impl ClientManager {
    /*
    Creates a new PlayerManager with a channel to send events to it
    This creates 2 tasks for sending/receiving on the actual websocket connection
    */
    async fn new(
        mut ws_send: WsSender,
        mut ws_rec: WsReceiver,
        game_lobby_send: mpsc::Sender<GameLobbyEvent>,
    ) -> (Self, mpsc::Sender<ClientEvent>) {
        let (event_sender, event_receiver) = mpsc::channel(8);
        let (packet_send, mut packet_send_listener) = mpsc::channel(8);
        let (packet_receive_writer, packet_receive) = mpsc::channel(8);
        //The websocket receiving demon
        tokio::spawn(async move {
            while let Some(packet) = ws_rec.next().await {
                if let Err(_) = packet_receive_writer.send(packet).await {
                    //If the receiving half of the channel is closed, stop listening for packets
                    break;
                }
            }
        });
        //The websocket sending demon
        tokio::spawn(async move {
            while let Some(packet) = packet_send_listener.recv().await {
                match packet {
                    PacketToClient::CloseConnection => {
                        if let Err(e) = ws_send.close().await {
                            error!("Error closing connection to client: {}", e);
                        }
                    }
                    _ => {
                        send_logging(&mut ws_send, packet).await;
                    }
                }
            }
        });
        (
            ClientManager {
                event_receive: event_receiver,
                packet_receive,
                packet_send,
                game_lobby_send,
            },
            event_sender,
        )
    }

    /*
    Runs the player manager
    */
    async fn run(&mut self) {
        loop {
            select! {
                event = self.event_receive.recv() => {
                    match event {
                        None => {
                            return; //TODO Better shutdown
                        }
                        Some(event) => {

                        }
                    }
                }
                packet = self.packet_receive.recv() => {
                    match packet {
                        None => {
                            return; //TODO Better shutdown
                        }
                        Some(packet) => {

                        }
                    }
                }
            }
        }
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
