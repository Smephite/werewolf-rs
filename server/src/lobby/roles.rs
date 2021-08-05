use std::sync::mpsc;

use anyhow::Error;
use async_trait::async_trait;
use werewolf_rs::game::Role;

use super::GameLobbyEvent;

/*
A trait for adding functionality to the Role type (from the shared code in werewolf-rs) that is only needed by the server
*/
#[async_trait]
pub trait ServerRole {
    async fn run_night_turn(&self, lobby_sender: mpsc::Sender<GameLobbyEvent>)
        -> Result<(), Error>;
}

#[async_trait]
impl ServerRole for Role {
    async fn run_night_turn(
        &self,
        lobby_sender: mpsc::Sender<GameLobbyEvent>,
    ) -> Result<(), Error> {
        match self {
            Role::Spectator => {}
            Role::Villager => {}
        }
        Ok(())
    }
}
