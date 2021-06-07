#![allow(incomplete_features)]
#![feature(specialization)]
pub mod server_network_manager;

use crate::server_network_manager::ServerNetworkManager;
use werewolf_rs::network_manager::NetworkManager;
use werewolf_rs::packets::packet::*;

fn main() {
    let manager = ServerNetworkManager {};

    let _ = manager.send_packet(&Packet::ToClient(ToClient::Ping("test!")));
}
