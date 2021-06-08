#[macro_use] extern crate log;
mod game_lobby;
mod lobby_manager;

use std::env;
use tokio::{net::TcpListener, runtime::Builder};

fn main() {
    let runtime = Builder::new_multi_thread().build().unwrap();
    runtime.block_on(async {
        let address =
            env::var("WEREWOLF_WEBSOCKET_ADDRESS").unwrap_or("127.0.0.1:8080".to_string());
        let listener = TcpListener::bind(address).await.unwrap();

        while let Ok((stream, _)) = listener.accept().await {
            match async_tungstenite::tokio::accept_async(stream).await {
                Err(_) => {}
                Ok(ws_stream) => {}
            }
        }
    });
}
