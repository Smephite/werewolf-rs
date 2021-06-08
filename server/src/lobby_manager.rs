use super::game_lobby::GameLobby;
use anyhow::Error;
use async_tungstenite::{
    tungstenite::Message,
    {tokio::TokioAdapter, WebSocketStream},
};
use futures::{SinkExt, StreamExt};
use rand::Rng;
use std::collections::HashMap;
use tokio::net::TcpStream;
use werewolf_rs::packet::{deserialize_packet, serialize_packet, PacketToClient, PacketToServer};

pub struct LobbyManager {
    lobbies: HashMap<usize, ()>,
}

impl LobbyManager {
    //Handles the given websocket connection and creates/joins a new lobby
    pub async fn handle_connection(&mut self, ws_stream: WebSocketStream<TokioAdapter<TcpStream>>) {
        let (ws_write, ws_read) = ws_stream.split();
        //Convert from/to "Packet"
        let ws_write = ws_write.with::<PacketToClient, _, _, Error>(|packet| async move {
            Ok::<_, Error>(Message::Text(serialize_packet(&packet)?))
        });
        let ws_read = ws_read.map(|msg| match msg {
            Ok(Message::Text(msg)) => deserialize_packet(&msg).unwrap_or(PacketToServer::Unknown),
            Ok(Message::Close(_)) => PacketToServer::CloseConnection,
            _ => PacketToServer::Unknown,
        });
        let lobby = GameLobby { ws_read, ws_write };
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
