use std::sync::{atomic::AtomicUsize, Arc};

use anyhow::Context;
use dashmap::DashMap;
use log::warn;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{net::TcpListener, sync::broadcast};
use ts_rs::TS;

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

#[derive(Debug)]
pub struct ClientInfo {
    username: String,
}

/// The state of the game server.
/// Stores all information about lobbies, games, and everthing else server related.
pub struct GameServerState {
    /// The id of the next player to connect to the game server.
    next_client_id: AtomicUsize,

    /// The id of the next created lobby
    next_lobby_id: AtomicUsize,

    /// The lobbies.
    lobbies: DashMap<LobbyId, Lobby>,

    /// Information about all clients connected to the game server.
    client_infos: DashMap<ClientId, ClientInfo>,

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
            client_infos: DashMap::new(),
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
    /// If there is no lobby with the given id, returns an error.
    /// If the lobby is full, returns an error.
    /// If the game in the lobby has already started, returns an error.
    /// Otherwise adds the client to the list of players in the lobby, notifies the other clients
    /// about the new player in the lobby, and returns the lobby's broadcase messages sender.
    pub fn join_lobby(
        &self,
        player_id: ClientId,
        lobby_id: LobbyId,
    ) -> Result<broadcast::Sender<ServerMessage>, JoinLobbyError> {
        let mut lobby = self
            .lobbies
            .get_mut(&lobby_id)
            .ok_or(JoinLobbyError::NoSuchLobby)?;
        if lobby.players_amount() >= MAX_PLAYERS_IN_LOBBY {
            return Err(JoinLobbyError::LobbyFull);
        }
        if lobby.state() != LobbyState::Waiting {
            return Err(JoinLobbyError::GameAlreadyStarted);
        }

        // get the username of the player.
        // it's safe to unwrap here because there's no way that the player is not in the client
        // infos list. that's because the only place we remove it from the list is in the
        // `ClientHandler::cleanup` function, but there is no way this function is called after
        // the cleanup.
        let username = self.client_infos.get(&player_id).unwrap().username.clone();

        lobby.add_player(player_id);

        // we can ignore the return value since we know it will be Ok(()), becuase the
        // lobby can't be empty otherwise it wouldn't exist, so we must still have listeners
        let _ = lobby
            .broadcast_messages_sender
            .send(ServerMessage::PlayerJoinedLobby(ExposedLobbyPlayerInfo {
                id: player_id,
                username,
            }));

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
            RemovePlayerFromLobbyResult::Ok => {
                // let the other clients know that this player left the lobby
                // we can ignore the return value since we know it will be Ok(()), becuase the
                // lobby isn't empty, so we still have listeners
                let _ = lobby
                    .broadcast_messages_sender
                    .send(ServerMessage::PlayerLeftLobby(player_id));
            }
            RemovePlayerFromLobbyResult::NewOwner(new_owner_id) => {
                // let the other clients know that this player left the lobby, and about the new
                // owner.
                //
                // we can ignore the return value since we know it will be Ok(()), becuase the
                // lobby isn't empty, so we still have listeners.
                let _ = lobby
                    .broadcast_messages_sender
                    .send(ServerMessage::OwnerLeftLobby { new_owner_id });
            }
            RemovePlayerFromLobbyResult::LobbyNowEmpty => {
                // the lobby is now empty, remove it
                self.lobbies.remove(&lobby_id);
            }
            RemovePlayerFromLobbyResult::PlayerWasntInLobby => {
                // no need to notify anyone because the player wasn't even in the lobby
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
                    players: lobby
                        .player_ids()
                        .filter_map(|player_id| {
                            Some(ExposedLobbyPlayerInfo {
                                id: player_id,
                                username: self.client_infos.get(&player_id)?.username.clone(),
                            })
                        })
                        .collect(),
                    owner_id: lobby.owner_id(),
                }
            })
            .collect()
    }

    /// Sets the username of the client with the given id.
    pub fn set_username(&self, client_id: ClientId, new_username: String) {
        let mut client_info = match self.client_infos.get_mut(&client_id) {
            Some(client_info) => client_info,
            None => {
                // if there is no user with the given id, just do nothing
                return;
            }
        };
        client_info.username = new_username;
    }

    /// Adds a new client to the list of connected clients, and generates a default username for it.
    pub fn add_client(&self, client_id: ClientId) {
        self.client_infos.insert(
            client_id,
            ClientInfo {
                username: format!("user{}", client_id),
            },
        );
    }

    /// Removes the client from the list of connected clients.
    pub fn remove_client(&self, client_id: ClientId) {
        self.client_infos.remove(&client_id);
    }

    /// Attempts to start the game in the given lobby, given the id of the client who requested to
    /// start the game and the id of the lobby.
    pub fn start_game(&self, requesting_client_id: ClientId, lobby_id: LobbyId) -> Result<(),StartGameError> {
        let mut lobby = self
            .lobbies
            .get_mut(&lobby_id)
            .ok_or(StartGameError::NoSuchLobby)?;

        // can only start the game if you are the owner
        if lobby.owner_id() != requesting_client_id{
            return Err(StartGameError::NotOwner);
        }

        // can only start the game if it's in the waiting state
        if lobby.state() != LobbyState::Waiting{
            return Err(StartGameError::GameAlreadyStarted)
        }

        lobby.start_game();

        Ok(())
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

#[derive(Debug, Error, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StartGameError {
    #[error("no such lobby")]
    NoSuchLobby,

    #[error("you are not the owner of this lobby")]
    NotOwner,

    #[error("the game in this lobby has already started")]
    GameAlreadyStarted,
}

/// The information about a lobby that is exposed to the clients.
#[derive(Debug, Serialize, Clone, TS)]
#[ts(export)]
pub struct ExposedLobbyInfo {
    pub name: String,

    #[ts(type = "number")]
    pub id: LobbyId,

    pub players: Vec<ExposedLobbyPlayerInfo>,

    #[ts(type = "number")]
    pub owner_id: ClientId,
}

/// The information about a lobby player that is exposed to the clients.
#[derive(Debug, Serialize, Clone, TS)]
#[ts(export)]
pub struct ExposedLobbyPlayerInfo {
    #[ts(type = "number")]
    id: ClientId,

    #[ts(type = "number")]
    username: String,
}
