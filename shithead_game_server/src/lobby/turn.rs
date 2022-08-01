use std::sync::Arc;

use set_timeout::CancellationToken;
use tokio::sync::Notify;

use crate::{
    lobby::{ClientId, LobbyId, GAME_SERVER_STATE, TURN_DURATION},
    util::TIMEOUT_SCHEDULER,
};

/// Represents a turn of a player in the game lobby.
#[derive(Debug)]
pub struct Turn {
    player_id: ClientId,
    next_turn_timeout_cancellation_token: CancellationToken,
}
impl Turn {
    pub fn new(lobby_id: LobbyId, player_id: ClientId) -> Self {
        let next_turn_timeout_cancellation_token =
            TIMEOUT_SCHEDULER.set_timeout(TURN_DURATION, async move {
                // if a timeout has occured, we must switch to the next turn
                GAME_SERVER_STATE.turn_timeout(lobby_id).await;
            });
        Self {
            player_id,
            next_turn_timeout_cancellation_token,
        }
    }

    /// The id of the player which this turn belongs to.
    pub fn player_id(&self) -> ClientId {
        self.player_id
    }

    /// Stops the next turn timer.
    pub async fn stop_next_turn_timer(self) {
        TIMEOUT_SCHEDULER.cancel_timeout(self.next_turn_timeout_cancellation_token);
    }
}
