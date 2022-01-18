use serde::{Deserialize, Serialize};

use crate::{
    game_server::{ClientId, ExposedLobbyInfo},
    lobby::LobbyId,
};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ServerMessage {
    ClientId(ClientId),
    Lobbies(Vec<ExposedLobbyInfo>),
    JoinLobby(LobbyId),
    Error(String),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "location", rename_all = "camelCase")]
pub enum ClickedCardLocation {
    Trash,
    #[serde(rename_all = "camelCase")]
    MyCards {
        card_index: u32,
    },
}
