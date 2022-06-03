mod util;

use shithead_game_server::TestClient;
use util::start_test_game_server;

#[tokio::test]
async fn perform_handshake() {
    start_test_game_server().await;

    let _ = TestClient::connect_and_perform_handshake().await;
}
