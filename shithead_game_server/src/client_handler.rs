use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use futures::{SinkExt, StreamExt};
use thiserror::Error;
use tokio::{net::TcpStream, sync::broadcast};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::{
    game_server::{ClientId, GameServerState},
    lobby::LobbyId,
    messages::{ClientMessage, ServerMessage},
};

#[derive(Debug, Error)]
#[error("the client sent an unknown message type: {0:?}")]
struct ClientSentUnknwonMsgType(Message);

/// A client handler, responsible for communicating with the client.
pub struct ClientHandler {
    websocket: WebSocketStream<TcpStream>,
    server_state: Arc<GameServerState>,
    broadcast_messages_sender: broadcast::Sender<ServerMessage>,
    username: String,
    client_id: ClientId,
    lobby_id: Option<LobbyId>,
}
impl ClientHandler {
    /// Handles a client by receiving messages from him and processing them, and by sending him
    /// broadcast messages.
    pub async fn handle(&mut self) -> anyhow::Result<()> {
        self.perform_handshake()
            .await
            .context("failed to perform handshake with client")?;

        let mut broadcast_messages_receiver = self.broadcast_messages_sender.subscribe();

        loop {
            tokio::select! {
                websocket_recv_result = self.websocket.next() =>{
                    // received a websocket message

                    // make sure we received an actual message and not None, which means the
                    // connection was closed.
                    let recv_result = match websocket_recv_result{
                        Some(recv_result) => {
                            recv_result
                        },
                        None => {
                            // connection closed, stop handling client
                            break
                        }
                    };

                    // check for errors, if no errors occured, proceed
                    let websocket_msg = recv_result.context("failed to receive message from client")?;

                    match websocket_msg {
                        Message::Text(text) => {
                            // parse the message
                            let msg: ClientMessage = serde_json::from_str(&text).context("failed to parse message")?;
                            self.handle_message(msg).await.context("failed to handle message from client")?;
                        }
                        Message::Close(_) => {
                            break;
                        }
                        _ => return Err(ClientSentUnknwonMsgType(websocket_msg).into()),
                    }
                },
                broadcast_msg_recv_result = broadcast_messages_receiver.recv() => {
                    // received a broadcast message
                    let broadcast_msg = broadcast_msg_recv_result.expect("the broadcast messages channel was closed");
                    self.send_message(&broadcast_msg).await.context("failed to send broadcast message to client")?;
                }
            }
        }
        Ok(())
    }

    /// Performs a handshake with this client, and synchronizes data with him.
    /// It sends the client its id, and a list of lobbies.
    async fn perform_handshake(&mut self) -> anyhow::Result<()> {
        // send the client its id
        self.send_message(&ServerMessage::ClientId(self.client_id))
            .await
            .context("failed to send the client its id")?;

        // send the client a list of exposed lobby information
        self.send_message(&ServerMessage::Lobbies(
            self.server_state.exposed_lobby_list(),
        ))
        .await
        .context("failed to send the list of lobbies to the client")?;

        Ok(())
    }

    /// Handles a message received from the client
    async fn handle_message(&mut self, msg: ClientMessage) -> anyhow::Result<()> {
        match msg {
            ClientMessage::Username(new_username) => {
                self.username = new_username;
            }
            ClientMessage::JoinLobby(lobby_id) => {
                match self.server_state.join_lobby(self.client_id, lobby_id) {
                    Ok(()) => {
                        self.lobby_id = Some(lobby_id);

                        // let the client know that he's now in the lobby
                        self.send_message(&ServerMessage::JoinLobby(lobby_id))
                            .await?;
                    }
                    Err(err) => {
                        // let the client know about the error that occured
                        self.send_message(&ServerMessage::Error(err.to_string()))
                            .await?;
                    }
                }
            }
            ClientMessage::CreateLobby { name } => {
                let new_lobby_id = self.server_state.create_lobby(name, self.client_id);
                self.lobby_id = Some(new_lobby_id);

                // let the client know that he's now in the lobby
                self.send_message(&ServerMessage::JoinLobby(new_lobby_id))
                    .await?;
            }
        }
        Ok(())
    }

    /// Sends a message to the client
    async fn send_message(&mut self, msg: &ServerMessage) -> anyhow::Result<()> {
        // serialize the message
        let serialized =
            serde_json::to_string(&msg).context("failed to serialize message to client")?;

        // send the message
        self.websocket
            .send(Message::Text(serialized))
            .await
            .context("failed to send websocket message to client")?;

        Ok(())
    }

    /// Sends a broadcast message to all clients, including the one handled by this handler.
    async fn send_broadcast_message(&mut self, msg: ServerMessage) {
        // we don't care if this fails becuase all it means is that there are no listeners, which
        // just means there aren't any clients, which is not really a problem.
        let _ = self.broadcast_messages_sender.send(msg);
    }
}

/// Handles a new client that had just connected to the game server's tcp listener.
pub async fn handle_client(
    server_state: Arc<GameServerState>,
    broadcast_messages_sender: broadcast::Sender<ServerMessage>,
    stream: TcpStream,
    addr: SocketAddr,
) -> anyhow::Result<()> {
    let websocket = tokio_tungstenite::accept_async(stream)
        .await
        .context("failed to accept websocket client {}")?;

    let id = server_state.next_client_id();

    let mut game_client = ClientHandler {
        websocket,
        broadcast_messages_sender,
        server_state,
        client_id: id,
        username: format!("user{}", id),
        lobby_id: None,
    };

    game_client.handle().await
}
