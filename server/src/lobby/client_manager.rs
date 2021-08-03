use std::collections::HashMap;

use super::GameLobbyEvent;
use crate::util::{generate_random_id, send_logging, WsReceiver, WsSender};
use futures::{SinkExt, StreamExt};
use tokio::{
    select,
    sync::{mpsc, oneshot},
};
use werewolf_rs::packet::{
    InteractionFollowup, InteractionRequest, InteractionResponse, PacketToClient, PacketToServer,
};

pub enum ClientEvent {
    /*Create an interaction and send the packet to the client.
    The interaction ID is sent back over the provided oneshot channel*/
    CreateInteraction(
        InteractionRequest,
        mpsc::Sender<InteractionResponse>,
        oneshot::Sender<u64>,
    ),
    //Send an interaction followup for the interaction with the given id
    FollowupInteraction(u64, InteractionFollowup),
    CloseInteraction(u64),
}

/*
A struct that manages the connection to one client in a game lobby.
It manages asynchronously sending and receiving packets from the client (using 2 additional tasks).
*/
pub struct ClientManager {
    packet_send: mpsc::Sender<PacketToClient>,
    packet_receive: mpsc::Receiver<PacketToServer>,
    event_receive: mpsc::Receiver<ClientEvent>,
    game_lobby_send: mpsc::Sender<GameLobbyEvent>,

    client_id: u64,
    interactions: HashMap<u64, mpsc::Sender<InteractionResponse>>,
}

impl ClientManager {
    /*
    Creates a new PlayerManager with a channel to send events to it
    */
    pub async fn new(
        lobby_id: u64,
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
            send_logging(
                &mut ws_send,
                PacketToClient::JoinedLobby {
                    lobby_id,
                    client_id,
                },
            )
            .await;
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
                client_id,
                interactions: HashMap::new(),
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
                //Receive an event (for example from the LobbyManager) and handle it
                event = self.event_receive.recv() => {
                    match event {
                        None => {
                            return;
                        }
                        Some(event) => {
                            match event {
                                ClientEvent::CreateInteraction(data, response_channel, id_oneshot) => {
                                    let interaction_id = generate_random_id(&self.interactions);
                                    self.interactions.insert(interaction_id, response_channel);
                                    id_oneshot.send(interaction_id).ok();
                                    self.packet_send.send(PacketToClient::InteractionRequest {
                                        interaction_id,
                                        data
                                    }).await.unwrap();
                                }
                                ClientEvent::FollowupInteraction(interaction_id, data) => {
                                    self.packet_send.send(PacketToClient::InteractionFollowup {
                                        interaction_id,
                                        data
                                    }).await.unwrap();
                                }
                                ClientEvent::CloseInteraction(interaction_id) => {
                                    self.packet_send.send(PacketToClient::InteractionClose { interaction_id }).await.unwrap();
                                }
                            }
                        }
                    }
                }
                //Receive a packet from the client and handle it
                packet = self.packet_receive.recv() => {
                    match packet {
                        None => {
                            //Notify the game lobby that the client lost its connection
                            self.game_lobby_send.send(GameLobbyEvent::ConnectionLost { client_id: self.client_id }).await.unwrap();
                            return;
                        }
                        Some(packet) => {
                            match packet {
                                PacketToServer::CloseConnection => {
                                    self.game_lobby_send.send(GameLobbyEvent::ConnectionLost { client_id: self.client_id }).await.unwrap();
                                    return;
                                }
                                PacketToServer::InteractionResponse { interaction_id, data } => {
                                    match self.interactions.get(&interaction_id) {
                                        None => {
                                            warn!("Received interaction response with unknown id: {}", interaction_id);
                                        }
                                        Some(channel) => {
                                            if let Err(e) = channel.send(data).await {
                                                error!("Unable to send back interaction response: {}", e);
                                            }
                                        }
                                    }
                                }
                                PacketToServer::StartGame => {
                                    self.game_lobby_send.send(GameLobbyEvent::StartGame { client_id: self.client_id} ).await.unwrap();
                                }
                                PacketToServer::Unknown | PacketToServer::JoinLobby(_) | PacketToServer::CreateNewLobby => {
                                    warn!("Received unknown/invalid packet from client in game lobby");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
