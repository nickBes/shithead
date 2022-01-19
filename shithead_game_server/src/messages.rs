use serde::{Deserialize, Serialize};
use typescript_type_def::TypeDef;

use crate::{
    game_server::{ClientId, ExposedLobbyInfo, ExposedLobbyPlayerInfo},
    lobby::LobbyId,
};

#[derive(Debug, Serialize, Clone, TypeDef)]
#[serde(rename_all = "camelCase")]
pub enum ServerMessage {
    ClientId(ClientId),
    Lobbies(Vec<ExposedLobbyInfo>),
    JoinLobby(LobbyId),
    Error(String),
    PlayerJoinedLobby(ExposedLobbyPlayerInfo),
    PlayerLeftLobby(ClientId),

    // reserved for future use
    LobbyOwnerChanged { new_owner_id: ClientId },

    // don't need the id of the previous owner because all the clients already know who the owner
    // is.
    OwnerLeftLobby { new_owner_id: ClientId },

    StartGame,

    ClickCard(ClickedCardLocation),
}

#[derive(Debug, Deserialize, Clone, TypeDef)]
#[serde(rename_all = "camelCase")]
pub enum ClientMessage {
    SetUsername(String),
    GetLobbies,
    JoinLobby(LobbyId),
    CreateLobby { lobby_name: String },
    StartGame,
    // ClickCard(ClickedCardLocation),
}

#[derive(Debug, Serialize, Deserialize, Clone, TypeDef)]
#[serde(rename_all = "camelCase")]
pub enum ClickedCardLocation {
    Trash,

    #[serde(rename_all = "camelCase")]
    MyCards {
        card_index: u32,
    },
}

/// The types to export as typescript bindings
type Bindings = (ServerMessage, ClientMessage);

#[test]
fn export_bindings() {
    let mut buf = Vec::new();
    typescript_type_def::write_definition_file::<_, Bindings>(&mut buf, Default::default())
        .unwrap();
    std::fs::write("bindings/bindings.ts", &buf).unwrap();
}
