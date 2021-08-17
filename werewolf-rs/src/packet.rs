use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    game::{CauseOfDeath, GameInfo},
    util::{InteractionId, LobbyId, PlayerId},
};

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
    JoinLobby(LobbyId),
    StartGame,
    //A response to an interaction that was created by the server. One interaction may be responded to several times, depending on its type
    InteractionResponse {
        interaction_id: InteractionId,
        data: InteractionResponse,
    },
    CloseConnection,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PacketToClient {
    UnknownLobbyId,
    JoinedLobby {
        lobby_id: LobbyId,
        client_id: PlayerId,
    },
    GameUpdate(GameInfo),
    PlayersDied(Vec<(PlayerId, CauseOfDeath)>),
    //The begin of an interaction (a series of packets that are linked by an ID)
    InteractionRequest {
        interaction_id: InteractionId,
        data: InteractionRequest,
    },
    //A followup message to an already existing interaction
    InteractionFollowup {
        interaction_id: InteractionId,
        data: InteractionFollowup,
    },
    InteractionClose {
        interaction_id: InteractionId,
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
        nominatable_player_ids: Vec<PlayerId>,
        can_vote: bool,
    },
}
#[derive(Serialize, Deserialize, Debug)]
pub enum InteractionResponse {
    NvNominate { nominated_player: Option<PlayerId> },
    NvVote { player_id: PlayerId },
}
#[derive(Serialize, Deserialize, Debug)]
pub enum InteractionFollowup {
    NvNewNomination {
        nominated_player: Option<PlayerId>,
        nominated_by: PlayerId,
    },
    NvNominationsFinished,
    NvVoteFinished {
        //(voter, vote) tuples for all votes
        votes: Vec<(PlayerId, PlayerId)>,
    },
}
