use std::sync::{atomic::AtomicUsize, Arc};

use anyhow::Context;
use dashmap::DashMap;
use log::warn;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{net::TcpListener, sync::broadcast};

use crate::{
    client_handler::handle_client,
    lobby::{Lobby, LobbyId, LobbyState, RemovePlayerFromLobbyResult, MAX_PLAYERS_IN_LOBBY},
    messages::ServerMessage,
};

const SERVER_BIND_ADDR: &str = "0.0.0.0:7522";
pub const BROADCAST_CHANNEL_CAPACITY: usize = 200;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(transparent)]
pub struct ClientId(usize);
impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The state of the game server.
/// Stores all information about lobbies, games, and everthing else server related.
pub struct GameServerState {
    /// The id of the next player to connect to the game server.
    next_client_id: AtomicUsize,

    /// The id of the next created lobby
    next_lobby_id: AtomicUsize,

    lobbies: DashMap<LobbyId, Lobby>,

    /// The channel for sending broadcast messages to all clients.
    pub broadcast_messages_sender: broadcast::Sender<ServerMessage>,
}
impl GameServerState {
    pub fn new() -> Self {
        let (broadcast_messages_sender, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        Self {
            next_client_id: AtomicUsize::new(0),
            next_lobby_id: AtomicUsize::new(0),
            lobbies: DashMap::new(),
            broadcast_messages_sender,
        }
    }

    /// Returns the id of the next player to connect to the game server.
    pub fn next_client_id(&self) -> ClientId {
        ClientId(
            self.next_client_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        )
    }

    /// Creates a new lobby with the given name and owner. Returns the id of the new lobby, and the
    /// lobby's broadcast messages sender.
    pub fn create_lobby(
        &self,
        name: String,
        owner_id: ClientId,
    ) -> (LobbyId, broadcast::Sender<ServerMessage>) {
        let lobby_id = LobbyId::from_raw(
            self.next_lobby_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        );
        let lobby = Lobby::new(name, owner_id);

        // save the broadcast_messages_sender before giving ownership of the lobby
        let broadcast_messages_sender = lobby.broadcast_messages_sender.clone();

        self.lobbies.insert(lobby_id, lobby);

        (lobby_id, broadcast_messages_sender)
    }

    /// Tries to add a player to a lobby.
    /// If there is no lobby with the given index, returns an error.
    /// If the lobby is full, returns an error.
    /// If the game in the lobby has already started, returns an error.
    /// Otherwise adds the client to the list of players in the lobby, and returns the lobby's
    /// broadcase messages sender.
    pub fn join_lobby(
        &self,
        player_id: ClientId,
        lobby_id: LobbyId,
    ) -> Result<broadcast::Sender<ServerMessage>, JoinLobbyError> {
        let lobby = self
            .lobbies
            .get(&lobby_id)
            .ok_or(JoinLobbyError::NoSuchLobby)?;
        if lobby.players_amount() >= MAX_PLAYERS_IN_LOBBY {
            return Err(JoinLobbyError::LobbyFull);
        }
        if lobby.state() != LobbyState::Waiting {
            return Err(JoinLobbyError::GameAlreadyStarted);
        }

        lobby.add_player(player_id);

        Ok(lobby.broadcast_messages_sender.clone())
    }

    /// Removes a player from a lobby.
    pub fn remove_player_from_lobby(&self, player_id: ClientId, lobby_id: LobbyId) {
        let mut lobby = match self.lobbies.get_mut(&lobby_id) {
            Some(lobby) => lobby,
            None => {
                // if the user is not really in this lobby, we don't need to do anything
                return;
            }
        };
        match lobby.remove_player(player_id) {
            RemovePlayerFromLobbyResult::Ok => {}
            RemovePlayerFromLobbyResult::NewOwner(new_owner) => {
                // TODO: notify all other players about the owner change.
            }
            RemovePlayerFromLobbyResult::LobbyNowEmpty => {
                // the lobby is now empty, remove it
                self.lobbies.remove(&lobby_id);
            }
        }
    }

    /// Returns a list of exposed information about each lobby.
    pub fn exposed_lobby_list(&self) -> Vec<ExposedLobbyInfo> {
        self.lobbies
            .iter()
            .map(|entry| {
                let lobby = entry.value();
                let lobby_id = *entry.key();
                ExposedLobbyInfo {
                    name: lobby.name().to_string(),
                    id: lobby_id,
                }
            })
            .collect()
    }
}

/// The main game server.
/// Responsible for accepting clients and spawning client handler tasks.
pub struct GameServer {
    /// The tcp listener, for accepting clients
    listener: TcpListener,

    /// The state of the server, passed down to the client handlers.
    state: Arc<GameServerState>,
}
impl GameServer {
    /// Creates a new game server
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(SERVER_BIND_ADDR)
                .await
                .context("failed to create tcp listener")?,
            state: Arc::new(GameServerState::new()),
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
            let _ = tokio::spawn(async move {
                if let Err(error) = handle_client(server_state, stream, addr).await {
                    warn!(
                        "failed to handle client with address {}, error: {:?}",
                        addr, error
                    );
                }
            });
        }
    }
}

#[derive(Debug, Error, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoinLobbyError {
    #[error("no such lobby")]
    NoSuchLobby,

    #[error("this lobby is full")]
    LobbyFull,

    #[error("the game in this lobby has already started")]
    GameAlreadyStarted,

    #[error("you are already in a lobby")]
    AlreadyInALobby,
}

/// The information about a lobby that is exposed to the clients.
#[derive(Debug, Serialize, Clone)]
pub struct ExposedLobbyInfo {
    pub name: String,
    pub id: LobbyId,
}
