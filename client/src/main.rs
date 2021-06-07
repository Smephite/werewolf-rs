#![allow(incomplete_features)]
#![feature(specialization)]
pub mod client_network_manager;

use crate::client_network_manager::ClientNetworkManager;
use werewolf_rs::packets::packet::*;
use werewolf_rs::network_manager::NetworkManager;

fn main() {
    let manager = ClientNetworkManager{};

    let _ = manager.send_packet(&Packet::ToServer(ToServer::Pong("test!")));
}
