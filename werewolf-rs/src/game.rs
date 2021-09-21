use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::util::PlayerId;

/*
The roles that a client in werewolf may have
*/
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Role {
    Spectator,
    Villager,
    Werewolf,
}

/*
The data that is associated to the role of a player. Note that this is usually not visible to everyone
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleData {
    Spectator,
    Villager,
    Werewolf,
}

/*
The information on a game that is visible to a specific player
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub players: HashMap<PlayerId, PlayerInfo>,
}

/*
The information on a player that is visible to the same or a different player
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub role_info: RoleInfo,
    pub is_alive: bool,
    pub is_lobby_host: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleInfo {
    Unknown,
    Known(Role),
    KnownData(RoleData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CauseOfDeath {
    Unknown,
    VillageVote,
    Werewolves,
}

impl Role {
    pub fn is_player(&self) -> bool {
        !matches!(self, Role::Spectator)
    }
    /*
    A list of roles that have to have finished their actions for this night, before this role's turn
    */
    pub fn dependencies_in_night(&self) -> Vec<Role> {
        match self {
            Self::Spectator => Vec::new(),
            Self::Villager => Vec::new(),
            Self::Werewolf => Vec::new(),
        }
    }
}

impl RoleData {
    pub fn new(role: &Role) -> Self {
        match role {
            Role::Spectator => Self::Spectator,
            Role::Villager => Self::Villager,
            Role::Werewolf => Self::Werewolf,
        }
    }

    pub fn get_role(&self) -> Role {
        match self {
            Self::Spectator => Role::Spectator,
            Self::Villager => Role::Villager,
            Self::Werewolf => Role::Werewolf,
        }
    }
}
