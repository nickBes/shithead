use anyhow::Context;
use game_server::GameServer;
use log::error;
use simple_logger::SimpleLogger;

mod cards;
mod client_handler;
mod game_server;
mod lobby;
mod messages;

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
