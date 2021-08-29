use super::ServerRole;
use async_trait::async_trait;

pub struct Villager;

#[async_trait]
impl ServerRole for Villager {
    async fn run_night_turn(&self, _: &tokio::sync::mpsc::Sender<crate::game::GameLobbyEvent>)
        -> Result<(), anyhow::Error> {
        Ok(())
    }
}