use crate::lobby::ClientId;
use crate::lobby::LobbyId;
use crate::lobby::GAME_SERVER_STATE;
use crate::lobby::TURN_DURATION;
use std::sync::Arc;
use tokio::sync::Notify;

/// A timer which waits for the next turn, and once the time is out, switches the turn.
#[derive(Debug)]
pub struct NextTurnTimer {
    task: tokio::task::JoinHandle<()>,
    notify: Arc<Notify>,
}
impl NextTurnTimer {
    pub async fn stop(self) {
        // notifying this `Notify` object will notify the task because the task is waiting to
        // receive a notification on it, and when the task will receive the notification it will
        // stop.
        self.notify.notify_one();
        self.task.await.unwrap();
    }
}

/// Represents a turn of a player in the game lobby.
#[derive(Debug)]
pub struct Turn {
    player_id: ClientId,
    next_turn_timer: NextTurnTimer,
}
impl Turn {
    pub fn new(lobby_id: LobbyId, player_id: ClientId) -> Self {
        let notify = Arc::new(Notify::new());
        let notify_clone = Arc::clone(&notify);
        let next_turn_timer = NextTurnTimer {
            task: tokio::spawn(async move {
                match tokio::time::timeout(TURN_DURATION, notify_clone.notified()).await {
                    Ok(()) => {
                        // we got a notification, which means the client played its turn before the
                        // time was up, so we can just stop the next turn timer.
                        return;
                    }
                    Err(_) => {
                        // if a timeout has occured, we must switch to the next turn
                        GAME_SERVER_STATE.turn_timeout(lobby_id).await;
                    }
                }
            }),
            notify,
        };
        Self {
            player_id,
            next_turn_timer,
        }
    }

    /// The id of the player which this turn belongs to.
    pub fn player_id(&self) -> ClientId {
        self.player_id
    }

    /// Stops the next turn timer.
    pub async fn stop_next_turn_timer(self) {
        self.next_turn_timer.stop().await
    }
}
