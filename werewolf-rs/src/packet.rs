use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::game::GameInfo;

pub fn serialize_packet<P: Serialize>(packet: &P) -> Result<String> {
    let raw: String = serde_json::ser::to_string(packet)?;
    Ok(raw)
}

pub fn deserialize_packet<'a, P: Deserialize<'a>>(raw: &'a str) -> Result<P> {
    let packet: P = serde_json::from_str(raw)?;
    Ok(packet)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PacketToServer {
    CreateNewLobby,
    JoinLobby(u64),
    StartGame,
    //A response to an interaction that was created by the server. One interaction may be responded to several times, depending on its type
    InteractionResponse {
        interaction_id: u64,
        data: InteractionResponse,
    },
    CloseConnection,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PacketToClient {
    UnknownLobbyId,
    JoinedLobby {
        lobby_id: u64,
        client_id: u64,
    },
    GameUpdate(GameInfo),
    //The begin of an interaction (a series of packets that are linked by an ID)
    InteractionRequest {
        interaction_id: u64,
        data: InteractionRequest,
    },
    //The begin of an interaction (a series of packets that are linked by an ID)
    InteractionFollowup {
        interaction_id: u64,
        data: InteractionFollowup,
    }, //A followup message to an already existing interaction
    InteractionClose {
        interaction_id: u64,
    },
    Ping(String),
    CloseConnection,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InteractionRequest {}
#[derive(Serialize, Deserialize, Debug)]
pub enum InteractionResponse {}
#[derive(Serialize, Deserialize, Debug)]
pub enum InteractionFollowup {}
