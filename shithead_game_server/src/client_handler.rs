use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use futures::{SinkExt, StreamExt};
use thiserror::Error;
use tokio::{net::TcpStream, sync::broadcast};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::{
    game_server::GameServerState,
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
    id: usize,
}
impl ClientHandler {
    /// Handles a client by receiving messages from him and processing them, and by sending him
    /// broadcast messages.
    pub async fn handle(&mut self) -> anyhow::Result<()> {
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

    /// Handles a message received from the client
    async fn handle_message(&mut self, msg: ClientMessage) -> anyhow::Result<()> {
        match msg {
            ClientMessage::Msg(msg) => self.send_broadcast_message(ServerMessage::Msg(msg)).await,
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

    let mut game_client = ClientHandler {
        websocket,
        broadcast_messages_sender,
        id: server_state
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        server_state,
    };

    game_client.handle().await
}
