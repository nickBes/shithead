use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    game_server::{ClientId, ExposedLobbyInfo, ExposedLobbyPlayerInfo},
    lobby::LobbyId,
};

#[derive(Debug, Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ServerMessage {
    ClientId(#[ts(type = "number")] ClientId),
    Lobbies(Vec<ExposedLobbyInfo>),
    JoinLobby(#[ts(type = "number")] LobbyId),
    Error(String),
    PlayerJoinedLobby(ExposedLobbyPlayerInfo),
    PlayerLeftLobby(#[ts(type = "number")] ClientId),

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

    ClickCard(ClickedCardLocation),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ClientMessage {
    Username(String),
    GetLobbies,
    JoinLobby(LobbyId),
    CreateLobby { name: String },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum InLobbyClientMessage {
    ClickCard(ClickedCardLocation),
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
