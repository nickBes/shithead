use serde::{Deserialize, Serialize};

use crate::game_server::ClientId;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ServerMessage {
    ClientId(ClientId),
    ClickCard(ClickedCardLocation),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ClientMessage {
    ClickCard(ClickedCardLocation),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "location", rename_all = "camelCase")]
pub enum ClickedCardLocation {
    Deck,
    #[serde(rename_all = "camelCase")]
    MyCards{ card_index: u32 },
}
