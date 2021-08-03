/*
The roles that a client in werewolf may have
*/
pub enum Role {
    Spectator,
}

/*
The data that is associated to the role of a player. Note that this is usually not visible to everyone
*/
pub enum RoleData {
    Spectator,
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
        }
    }
}

impl RoleData {
    pub fn get_role(&self) -> Role {
        match self {
            Self::Spectator => Role::Spectator,
        }
    }
}
