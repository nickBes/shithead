use anyhow::Context;
use game_server::GameServer;
use log::error;

mod cards;
mod client_handler;
mod game_server;
mod lobby;
mod messages;
mod util;

async fn run() -> anyhow::Result<()> {
    let mut game_server = GameServer::new()
        .await
        .context("failed to create game server")?;
    game_server.start().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    if let Err(error) = run().await {
        error!("{:?}", error);
    }
}
