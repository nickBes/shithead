use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    game_server::{ClientId, ExposedLobbyInfo, ExposedLobbyPlayerInfo},
    lobby::LobbyId,
};

#[derive(Debug, Serialize, Clone, TS)]
#[serde(tag = "t", rename_all = "camelCase")]
#[ts(export)]
pub enum ServerMessage {
    ClientId {
        #[ts(type = "number")]
        id: ClientId,
    },
    Lobbies {
        lobbies: Vec<ExposedLobbyInfo>,
    },
    JoinLobby {
        #[ts(type = "number")]
        id: LobbyId,
    },
    Error {
        err: String,
    },
    PlayerJoinedLobby {
        player_info: ExposedLobbyPlayerInfo,
    },
    PlayerLeftLobby {
        #[ts(type = "number")]
        id: ClientId,
    },

    // reserved for future use
    #[serde(rename_all = "camelCase")]
    LobbyOwnerChanged {
        #[ts(type = "number")]
        new_owner_id: ClientId,
    },

    // don't need the id of the previous owner because all the clients already know who the owner
    // is.
    OwnerLeftLobby {
        #[ts(type = "number")]
        new_owner_id: ClientId,
    },

    StartGame,

    ClickCard {
        location: ClickedCardLocation,
    },
}

#[derive(Debug, Deserialize, Clone, TS)]
#[serde(tag = "t", rename_all = "camelCase")]
#[ts(export)]
pub enum ClientMessage {
    SetUsername {
        new_username: String,
    },
    GetLobbies,
    JoinLobby {
        #[ts(type = "number")]
        id: LobbyId,
    },
    CreateLobby {
        name: String,
    },
    StartGame,
    // ClickCard(ClickedCardLocation),
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(tag = "location", rename_all = "camelCase")]
#[ts(export)]
pub enum ClickedCardLocation {
    Trash,

    #[serde(rename_all = "camelCase")]
    MyCards {
        card_index: u32,
    },
}
