use parity_wordlist::random_phrase;
use shithead_game_server::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Options {
    CreateLobbies { amount: usize },
}

#[tokio::main]
async fn main() {
    let options = Options::from_args();

    match options {
        Options::CreateLobbies { amount } => {
            let mut clients = Vec::new();
            for _ in 0..amount {
                let (mut client, _) = TestClient::connect_and_perform_handshake().await;
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
