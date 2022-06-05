use serde::Serialize;

use crate::cards::CardId;

#[derive(Debug, Serialize)]
pub struct LobbyPlayer {
    pub cards_in_hand: Vec<CardId>,
    pub three_up_cards: Vec<CardId>,
    pub three_down_cards: Vec<CardId>,
}

impl LobbyPlayer {
    /// Creates a new lobby player without any cards.
    pub fn without_any_cards() -> Self {
        Self {
            cards_in_hand: Vec::new(),
            three_up_cards: Vec::new(),
            three_down_cards: Vec::new(),
        }
    }
}
