use std::time::Duration;

use crate::*;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub const TEST_CLIENT_TIMEOUT: Duration = Duration::from_secs(1);

/// A test client used for testing the game server.
pub struct TestClient {
    ws: WebSocketStream<TcpStream>,
}
impl TestClient {
    /// Creates a new client by connecting to the server.
    pub async fn connect_to_server() -> Self {
        async fn connect_no_timeout() -> TestClient {
            let (ws, _) = tokio_tungstenite::client_async(
                format!("ws://{}", SERVER_BIND_ADDR),
                TcpStream::connect(SERVER_BIND_ADDR)
                    .await
                    .expect("failed to connect to server"),
            )
            .await
            .expect("failed to create websocket client");
            TestClient { ws }
        }
        tokio::time::timeout(TEST_CLIENT_TIMEOUT, connect_no_timeout())
            .await
            .expect("timeout while connecting to server")
    }

    /// Creates a new client by connecting to the server and validates the server's handshake.
    pub async fn connect_and_perform_handshake() -> (Self, HandshakeClientInfo) {
        let mut client = TestClient::connect_to_server().await;
        let client_info = client.perform_handshake().await;
        (client, client_info)
    }

    /// Receives a message from the server
    pub async fn recv_indefinitely(&mut self) -> ServerMessage {
        let ws_msg = self
            .ws
            .next()
            .await
            .expect("the stream was closed")
            .expect("failed to receive websocket message from server");
        match ws_msg {
            Message::Text(text) => {
                serde_json::from_str(&text).expect("failed to parse server message as json")
            }
            _ => {
                panic!(
                    "received an unexpected message from the server: {:?}",
                    ws_msg
                );
            }
        }
    }

    /// Receives a message from the server, panicking in case of a timeout.
    pub async fn recv(&mut self) -> ServerMessage {
        tokio::time::timeout(TEST_CLIENT_TIMEOUT, self.recv_indefinitely())
            .await
            .expect("timeout while receiving message from server")
    }

    /// Sends a message to the server
    pub async fn send(&mut self, msg: ClientMessage) {
        let text = serde_json::to_string(&msg).expect("failed to serialize message");
        self.ws
            .send(Message::Text(text))
            .await
            .expect("failed to send websocket message to server");
    }

    /// Performs a handshake with the game server, and returns the client's [`HandshakeClientInfo`] sent by
    /// the server.
    pub async fn perform_handshake(&mut self) -> HandshakeClientInfo {
        let client_id_msg = self.recv().await;
        let client_id = match client_id_msg {
            ServerMessage::Handshake(client_info) => client_info,
            _ => panic!(
                "expected a ClientId message during the handshake, instead got: {:?}",
                client_id_msg
            ),
        };

        client_id
    }
}
