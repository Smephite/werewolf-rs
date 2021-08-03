
use anyhow::Error;
use tokio::sync::{broadcast, mpsc};

use super::GameLobbyEvent;

/*
A struct that handles the game logic.
This does not contain the game state, as that is handled by the GameLobby
*/
pub struct GameRunner {
    lobby_sender: mpsc::Sender<GameLobbyEvent>,
    game_cancel: broadcast::Receiver<()>
}

impl GameRunner {
    /*
    Creates a new game runner and a broadcast sender that can be used to cancel the game (by stopping all created tasks at their next .await)
    */
    pub fn new(lobby_sender: mpsc::Sender<GameLobbyEvent>, game_cancel: broadcast::Receiver<()>) -> Self {
        GameRunner { 
            lobby_sender,
            game_cancel
        }
    }

    pub async fn start(mut self) {
        tokio::spawn(async move {
            if let Err(e) = self.run().await {
                error!("Error running game: {:?}", e);
            }
        });
    }

    async fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
