use futures::Stream;
use futures::Sink;
use werewolf_rs::packets::packet::{
    PacketToClient,
    PacketToServer,
};

pub struct GameLobby<WsWrite: Sink<PacketToClient>, WsRead: Stream<Item=PacketToServer>> {
    //ws_write: WsWrite,
    pub ws_read: WsRead,
    pub ws_write: WsWrite
}
