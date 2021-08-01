use std::{collections::HashMap, pin::Pin};

use anyhow::Error;
use futures::{Sink, SinkExt, Stream};
use rand::Rng;
use werewolf_rs::packet::{PacketToClient, PacketToServer};

pub type WsSender = Pin<Box<dyn Sink<PacketToClient, Error = Error> + Send + Sync>>;
pub type WsReceiver = Pin<Box<dyn Stream<Item = PacketToServer> + Send + Sync>>;

pub async fn send_logging(sender: &mut WsSender, data: PacketToClient) {
    if let Err(e) = sender.send(data).await {
        error!("Error sending websocket packet: {:?}", e);
    }
}

pub fn generate_random_id<V>(used_ids: &HashMap<u64, V>) -> u64 {
    loop {
        let id: u64 = rand::thread_rng().gen();
        if !used_ids.contains_key(&id) {
            return id;
        }
    }
}
