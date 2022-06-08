mod util;

use parity_wordlist::random_phrase;
use shithead_game_server::{
    ClientId, ClientMessage, ExposedLobbyPlayerInfo, LobbyId, ServerMessage, TestClient,
};
use util::start_test_game_server;

#[tokio::test]
async fn create_join_lobby() {
    start_test_game_server().await;

    let (mut creating_client, creating_client_handshake_info) =
        TestClient::connect_and_perform_handshake().await;
    creating_client
        .send(ClientMessage::CreateLobby {
            lobby_name: random_phrase(5),
        })
        .await;
    assert_eq!(
        creating_client.recv().await,
        ServerMessage::JoinLobby {
            lobby_id: LobbyId::from_raw(0),
            players: vec![]
        }
    );

    let (mut joining_client, joining_client_handshake_info) =
        TestClient::connect_and_perform_handshake().await;
    joining_client
        .send(ClientMessage::JoinLobby(LobbyId::from_raw(0)))
        .await;
    assert_eq!(
        joining_client.recv().await,
        ServerMessage::JoinLobby {
            lobby_id: LobbyId::from_raw(0),
            players: vec![ExposedLobbyPlayerInfo {
                id: ClientId::from_raw(0),
                username: creating_client_handshake_info.username
            }]
        }
    );

    assert_eq!(
        creating_client.recv().await,
        ServerMessage::PlayerJoinedLobby(ExposedLobbyPlayerInfo {
            id: ClientId::from_raw(1),
            username: joining_client_handshake_info.username,
        })
    );
}
