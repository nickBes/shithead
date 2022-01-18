use std::sync::{atomic::AtomicUsize, Arc};

use anyhow::Context;
use log::warn;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, sync::broadcast};

use crate::{client_handler::handle_client, messages::ServerMessage};

const SERVER_BIND_ADDR: &str = "0.0.0.0:7522";
const BROADCAST_CHANNEL_CAPACITY: usize = 200;

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(usize);

/// The state of the game server.
/// Stores all information about lobbies, games, and everthing else server related.
pub struct GameServerState {
    /// The id of the next player to connect to the game server.
    next_client_id: AtomicUsize,
}
impl GameServerState {
    pub fn new() -> Self {
        Self {
            next_client_id: AtomicUsize::new(0),
        }
    }

    pub fn next_client_id(&self) -> ClientId {
        ClientId(
            self.next_client_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        )
    }
}

/// The main game server.
/// Responsible for accepting clients and spawning client handler tasks.
pub struct GameServer {
    /// The tcp listener, for accepting clients
    listener: TcpListener,

    /// The state of the server, passed down to the client handlers.
    state: Arc<GameServerState>,

    /// The channel for sending broadcast messages to all clients.
    broadcast_messages_sender: broadcast::Sender<ServerMessage>,
}
impl GameServer {
    /// Creates a new game server
    pub async fn new() -> anyhow::Result<Self> {
        let (broadcast_messages_sender, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        Ok(Self {
            listener: TcpListener::bind(SERVER_BIND_ADDR)
                .await
                .context("failed to create tcp listener")?,
            state: Arc::new(GameServerState::new()),
            broadcast_messages_sender,
        })
    }

    /// Starts accepting clients and spawning client handlers for them
    pub async fn start(&mut self) -> anyhow::Result<()> {
        loop {
            let (stream, addr) = match self.listener.accept().await {
                Ok(res) => res,
                Err(error) => {
                    warn!("failed to accept client: {:?}", error);
                    continue;
                }
            };
            let server_state = Arc::clone(&self.state);
            let broadcast_messages_sender = self.broadcast_messages_sender.clone();
            let _ = tokio::spawn(async move {
                if let Err(error) =
                    handle_client(server_state, broadcast_messages_sender, stream, addr).await
                {
                    warn!(
                        "failed to handle client with address {}, error: {:?}",
                        addr, error
                    );
                }
            });
        }
    }
}
