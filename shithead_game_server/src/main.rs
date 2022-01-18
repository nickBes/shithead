use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use futures::{SinkExt, StreamExt};
use log::{error, warn};
use simple_logger::SimpleLogger;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

const SERVER_BIND_ADDR: &str = "0.0.0.0:7522";

/// The state of the game server.
/// Stores all information about lobbies, games, and everthing else server related.
pub struct GameServerState {}
impl GameServerState {
    pub fn new() -> Self {
        Self {}
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
                if let Err(error) = client_handler(server_state, stream, addr).await {
                    warn!("failed to handle client: {:?}", error);
                }
            });
        }
    }
}

async fn client_handler(
    server_state: Arc<GameServerState>,
    stream: TcpStream,
    addr: SocketAddr,
) -> anyhow::Result<()> {
    println!("client handler, addr: {}", addr);
    let mut websocket = tokio_tungstenite::accept_async(stream)
        .await
        .context("failed to accept websocket client")?;
    println!("done handling client");
    while let Some(recv_result) = websocket.next().await {
        let msg = recv_result.context("failed to receive message from client")?;
        println!("received msg from client: {}", msg);
        websocket
            .send(Message::text(format!("you said: {}", msg)))
            .await
            .context("failed to send message to client")?;
    }
    println!("done handling client");
    Ok(())
}

async fn run() -> anyhow::Result<()> {
    let mut game_server = GameServer::new()
        .await
        .context("failed to create game server")?;
    game_server.start().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        // display timestamps using utc time. using this because there's a problem on my pc which
        // doesn't allow using local time.
        .with_utc_timestamps()
        // only show log messages with level Info or higher (Warning, Error).
        .with_level(log::LevelFilter::Info)
        .init()
        .expect("failed to initialize logger");

    if let Err(error) = run().await {
        error!("{:?}", error);
    }
}
