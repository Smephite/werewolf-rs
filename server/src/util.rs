use anyhow::Error;
use futures::{Sink, Stream};
use werewolf_rs::packet::{PacketToClient, PacketToServer};


pub type WsSender = Box<dyn Sink<PacketToClient, Error=Error> + Send + Sync>;
pub type WsReceiver = Box<dyn Stream<Item=PacketToServer> + Send + Sync>;