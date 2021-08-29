use super::ServerRole;
use async_trait::async_trait;

pub struct Spectator;

#[async_trait]
impl ServerRole for Spectator {
    async fn run_night_turn(&self, _: &tokio::sync::mpsc::Sender<crate::game::GameLobbyEvent>)
        -> Result<(), anyhow::Error> {
        Ok(())
    }
}