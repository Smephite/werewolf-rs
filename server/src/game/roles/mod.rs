mod spectator;
mod villager;

use tokio::sync::mpsc;
use anyhow::Error;
use async_trait::async_trait;
use werewolf_rs::game::Role;

use super::GameLobbyEvent;

/*
A trait for the server logic of a role
*/
#[async_trait]
pub trait ServerRole {
    async fn run_night_turn(&self, lobby_sender: &mpsc::Sender<GameLobbyEvent>)
        -> Result<(), Error>;
}

/*
The purpose of this trait is to delegate functions called on the Role enum to their implementation.
This enables each role to be contained in its own file with a zero-sized struct implementing the ServerRole trait
*/
#[async_trait]
pub trait ServerRoleDelegator {
    fn get_implementor(&self) -> Box<dyn ServerRole + Send + Sync>;
    async fn run_night_turn(&self, lobby_sender: &mpsc::Sender<GameLobbyEvent>)
        -> Result<(), Error> {
            self.get_implementor().run_night_turn(lobby_sender).await
        }
}

#[async_trait]
impl ServerRoleDelegator for Role {
    fn get_implementor(&self) -> Box<dyn ServerRole + Send + Sync> {
        match self {
            Role::Spectator => Box::new(spectator::Spectator),
            Role::Villager => Box::new(villager::Villager)
        }
    }
}
