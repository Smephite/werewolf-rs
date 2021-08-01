#[macro_use]
extern crate log;
mod lobby;
mod lobby_manager;
mod util;

use anyhow::Error;
use async_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use lobby_manager::LobbyManager;
use std::env;
use tokio::{net::TcpListener, runtime::Builder, sync::mpsc};
use util::send_logging;
use werewolf_rs::packet::{deserialize_packet, serialize_packet, PacketToClient, PacketToServer};

fn main() {
    let runtime = Builder::new_multi_thread().build().unwrap();
    if let Err(e) = runtime.block_on(run_server()) {
        error!("Error while running websocket server: {:?}", e);
    }
}

async fn run_server() -> Result<(), Error> {
    let mut lobby_manager = LobbyManager::new();
    let (lobby_send, lobby_rec) = mpsc::channel::<lobby_manager::LobbyManagerEvent>(8);
    let lobby_send_cloned = lobby_send.clone();
    tokio::spawn(async move {
        lobby_manager.run(lobby_rec, lobby_send_cloned).await;
    });

    let address = env::var("WEREWOLF_WEBSOCKET_ADDRESS").unwrap_or("127.0.0.1:8080".to_string());
    let listener = TcpListener::bind(address).await?;

    while let Ok((stream, _)) = listener.accept().await {
        match async_tungstenite::tokio::accept_async(stream).await {
            Err(_) => {}
            Ok(ws_stream) => {
                let lobby_send = lobby_send.clone();
                tokio::spawn(async move {
                    let (ws_write, ws_read) = ws_stream.split();
                    //Convert from/to Packet
                    let ws_write = Box::pin(ws_write.with::<PacketToClient, _, _, Error>(
                        |packet| async move {
                            Ok::<_, Error>(Message::Text(serialize_packet(&packet)?))
                        },
                    ));
                    let ws_read = ws_read.map(|msg| match msg {
                        Ok(Message::Text(msg)) => {
                            deserialize_packet(&msg).unwrap_or(PacketToServer::Unknown)
                        }
                        Ok(Message::Close(_)) => PacketToServer::CloseConnection,
                        _ => PacketToServer::Unknown,
                    });
                    let mut ws_write: util::WsSender = Box::pin(ws_write);
                    let mut ws_read: util::WsReceiver = Box::pin(ws_read);
                    //Decide what to do with the connection based on the first received message
                    match ws_read.next().await {
                        Some(PacketToServer::CreateNewLobby) => {
                            lobby_send
                                .send(lobby_manager::LobbyManagerEvent::CreateNewLobby {
                                    ws_read,
                                    ws_write,
                                })
                                .await?;
                            Ok::<(), Error>(())
                        }
                        Some(PacketToServer::JoinLobby(lobby_id)) => {
                            lobby_send
                                .send(lobby_manager::LobbyManagerEvent::JoinLobby {
                                    ws_read,
                                    ws_write,
                                    lobby_id: lobby_id,
                                })
                                .await?;
                            Ok(())
                        }
                        Some(_) => {
                            send_logging(&mut ws_write, PacketToClient::ReceivedInvalidData).await;
                            Ok(())
                        }
                        None => Ok(()),
                    }
                });
            }
        }
    }
    Ok(())
}
