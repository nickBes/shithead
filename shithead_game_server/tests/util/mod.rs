use shithead_game_server::GameServer;

pub async fn start_test_game_server(){
    let mut game_server = GameServer::new().await.expect("failed to create game server");
    tokio::spawn(async move{
        game_server.start().await.expect("game server error");
    });
}
