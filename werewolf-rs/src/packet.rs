use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::game::{CauseOfDeath, GameInfo};

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
    PlayersDied(Vec<(u64, CauseOfDeath)>),
    //The begin of an interaction (a series of packets that are linked by an ID)
    InteractionRequest {
        interaction_id: u64,
        data: InteractionRequest,
    },
    //A followup message to an already existing interaction
    InteractionFollowup {
        interaction_id: u64,
        data: InteractionFollowup,
    },
    InteractionClose {
        interaction_id: u64,
    },
    Ping(String),
    CloseConnection,
    Unknown,
}

/*
The data that can be part of an interation. The interactions are:

- NominationVote (Nv)
    Each client can nominate a player (but doesn't have to).
    Afterwards, everyone can vote for one of the nominated players.
    There has to be at least one nomination before anyone can choose to nominate no one.
*/
#[derive(Serialize, Deserialize, Debug)]
pub enum InteractionRequest {
    NvBegin {
        nominatable_player_ids: Vec<u64>,
        can_vote: bool,
    },
}
#[derive(Serialize, Deserialize, Debug)]
pub enum InteractionResponse {
    NvNominate { nominated_player: Option<u64> },
    NvVote { player_id: u64 },
}
#[derive(Serialize, Deserialize, Debug)]
pub enum InteractionFollowup {
    NvNewNomination {
        nominated_player: Option<u64>,
        nominated_by: u64,
    },
    NvNominationsFinished,
    NvVoteFinished {
        //(voter, vote) tuples for all votes
        votes: Vec<(u64, u64)>,
    },
}
