use anyhow::Context;
use futures::{SinkExt, StreamExt};
use parity_wordlist::random_phrase;
use shithead_game_server::*;
use structopt::StructOpt;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

#[derive(Debug, StructOpt)]
pub enum Options {
    CreateLobbies { amount: usize },
}

struct TestClient {
    ws: WebSocketStream<TcpStream>,
}
impl TestClient {
    /// Creates a new client by connecting to the server.
    pub async fn connect_to_server() -> Self {
        let (ws, _) = tokio_tungstenite::client_async(
            format!("ws://{}", SERVER_BIND_ADDR),
            TcpStream::connect(SERVER_BIND_ADDR)
                .await
                .expect("failed to connect to server"),
        )
        .await
        .expect("failed to create websocket client");
        Self { ws }
    }

    /// Receives a message from the server
    pub async fn recv(&mut self) -> ServerMessage {
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

    /// Sends a message to the server
    pub async fn send(&mut self, msg: ClientMessage) {
        let text = serde_json::to_string(&msg).expect("failed to serialize message");
        self.ws
            .send(Message::Text(text))
            .await
            .expect("failed to send websocket message to server");
    }

    /// Validates the server's handshake messages, by making sure that it sends a `ClientId`
    /// message followed by a `Lobbies` message.
    pub async fn validate_handshake(&mut self) {
        let client_id_msg = self.recv().await;
        assert!(matches!(client_id_msg, ServerMessage::ClientId(_)));

        let lobbies_msg = self.recv().await;
        assert!(matches!(lobbies_msg, ServerMessage::Lobbies(_)));
    }
}

#[tokio::main]
async fn main() {
    let options = Options::from_args();

    match options {
        Options::CreateLobbies { amount } => {
            let mut clients = Vec::new();
            for _ in 0..amount {
                let mut client = TestClient::connect_to_server().await;
                client.validate_handshake().await;
                client
                    .send(ClientMessage::CreateLobby {
                        lobby_name: random_phrase(5),
                    })
                    .await;
                clients.push(client);
            }
            println!("keeping clients open");
            println!("to stop, press ctrl-c");
            tokio::signal::ctrl_c()
                .await
                .expect("failed to wait for ctrl-c");
        }
    }
}
