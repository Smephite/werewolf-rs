#![allow(incomplete_features)]
#![feature(specialization)]
mod client_network_manager;

use crate::client_network_manager::ClientNetworkManager;
use werewolf_rs::network_manager::NetworkManager;
use werewolf_rs::packets::packet::*;

mod view;
mod pages;

fn main() {
    let manager = ClientNetworkManager {};

    let _ = manager.send_packet(&Packet::ToServer(ToServer::Pong("test!")));
    view::start();
}
