use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typescript_type_def::TypeDef;

use crate::{
    cards::{Card, CardId},
    game_server::{ClientId, ExposedLobbyInfo, ExposedLobbyPlayerInfo},
    lobby::LobbyId,
};

#[derive(Debug, Serialize, Deserialize, Clone, TypeDef, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ServerMessage {
    Handshake(HandshakeClientInfo),
    Lobbies(Vec<ExposedLobbyInfo>),
    JoinLobby {
        lobby_id: LobbyId,
        players: Vec<ExposedLobbyPlayerInfo>,
    },
    Error(String),
    PlayerJoinedLobby(ExposedLobbyPlayerInfo),
    PlayerLeftLobby(ClientId),

    // reserved for future use
    LobbyOwnerChanged {
        new_owner_id: ClientId,
    },

    // don't need the id of the previous owner because all the clients already know who the owner
    // is.
    OwnerLeftLobby {
        new_owner_id: ClientId,
    },

    StartGame,

    CardsInHand(Vec<CardId>),

    MovePlayerCardFromThreeUpToHand {
        up_three_card_index: usize,
        player_id: ClientId,
    },

    MovePlayerCardFromHandToThreeUp {
        hand_card_index: usize,
        player_id: ClientId,
    },

    TheeUpSelectionDone {
        three_up_cards_of_modified_players: HashMap<ClientId, Vec<CardId>>,
    },

    GiveTrash(ClientId),

    Turn(ClientId),
}

#[derive(Debug, Serialize, Deserialize, Clone, TypeDef)]
#[serde(rename_all = "camelCase")]
pub enum ClientMessage {
    SetUsername(String),
    GetLobbies,
    JoinLobby(LobbyId),
    LeaveLobby,

    #[serde(rename_all = "camelCase")]
    CreateLobby {
        lobby_name: String,
    },

    GetCardsInHand,

    StartGame,

    ClickCard(ClickedCardLocation),
}

#[derive(Debug, Serialize, Deserialize, Clone, TypeDef, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HandshakeClientInfo {
    pub id: ClientId,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TypeDef, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ClickedCardLocation {
    Trash,

    #[serde(rename_all = "camelCase")]
    FromCardsInHand {
        card_index: usize,
    },

    FromThreeUpCards {
        card_index: usize,
    },

    FromThreeDownCards {
        card_index: usize,
    },
}

#[test]
fn export_bindings() {
    /// The types to export as typescript bindings
    type Bindings = (ServerMessage, ClientMessage, Card);

    let mut buf = Vec::new();
    typescript_type_def::write_definition_file::<_, Bindings>(&mut buf, Default::default())
        .unwrap();
    std::fs::write("bindings/bindings.ts", &buf).unwrap();
}
