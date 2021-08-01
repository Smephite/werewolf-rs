use super::GameLobbyEvent;
use crate::util::{send_logging, WsReceiver, WsSender};
use futures::{SinkExt, StreamExt};
use tokio::{select, sync::mpsc};
use werewolf_rs::packet::{PacketToClient, PacketToServer};

pub enum ClientEvent {}

pub struct ClientManager {
    packet_send: mpsc::Sender<PacketToClient>,
    packet_receive: mpsc::Receiver<PacketToServer>,
    event_receive: mpsc::Receiver<ClientEvent>,
    game_lobby_send: mpsc::Sender<GameLobbyEvent>,

    client_id: u64
}

impl ClientManager {
    /*
    Creates a new PlayerManager with a channel to send events to it
    This creates 2 tasks for sending/receiving on the actual websocket connection
    */
    pub async fn new(
        client_id: u64,
        mut ws_send: WsSender,
        mut ws_rec: WsReceiver,
        game_lobby_send: mpsc::Sender<GameLobbyEvent>,
    ) -> (Self, mpsc::Sender<ClientEvent>) {
        let (event_sender, event_receiver) = mpsc::channel(8);
        let (packet_send, mut packet_send_listener) = mpsc::channel(8);
        let (packet_receive_writer, packet_receive) = mpsc::channel(8);
        //The websocket receiving daemon
        tokio::spawn(async move {
            while let Some(packet) = ws_rec.next().await {
                if let Err(_) = packet_receive_writer.send(packet).await {
                    //If the receiving half of the channel is closed, stop listening for packets
                    break;
                }
            }
        });
        //The websocket sending daemon
        tokio::spawn(async move {
            while let Some(packet) = packet_send_listener.recv().await {
                match packet {
                    PacketToClient::CloseConnection => {
                        if let Err(e) = ws_send.close().await {
                            error!("Error closing connection to client: {}", e);
                        }
                        break;
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
                client_id
            },
            event_sender,
        )
    }

    /*
    Runs the player manager. This handles all incoming packets from the client, as well as events/requests sent to it over its channel
    */
    pub async fn run(&mut self) {
        loop {
            select! {
                event = self.event_receive.recv() => {
                    match event {
                        None => {
                            return;
                        }
                        Some(event) => {

                        }
                    }
                }
                packet = self.packet_receive.recv() => {
                    match packet {
                        None => {
                            //Notify the game lobby that the client lost its connection
                            self.game_lobby_send.send(GameLobbyEvent::ConnectionLost { client_id: self.client_id }).await.unwrap();
                        }
                        Some(packet) => {

                        }
                    }
                }
            }
        }
    }
}
