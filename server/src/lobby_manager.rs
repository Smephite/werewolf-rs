use super::game_lobby::GameLobby;

use async_tungstenite::{tokio::TokioAdapter, WebSocketStream};
use futures::stream::StreamExt;
use rand::Rng;
use std::collections::HashMap;
use tokio::net::TcpStream;

pub struct LobbyManager {
    lobbies: HashMap<usize, GameLobby>,
}

impl LobbyManager {
    //Handles the given websocket connection and creates/joins a new lobby
    pub async fn handle_connection(&mut self, ws_stream: WebSocketStream<TokioAdapter<TcpStream>>) {
        let (ws_write, ws_read) = ws_stream.split();
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
