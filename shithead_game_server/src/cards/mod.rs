mod cards_by_id_cache;
mod deck;

pub use cards_by_id_cache::*;
pub use deck::*;

use serde::{Deserialize, Serialize};
use strum_macros::{EnumCount, EnumIter};
use typescript_type_def::TypeDef;

pub const JOKERS_AMOUNT: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TypeDef)]
#[serde(transparent)]
pub struct CardId(usize);
impl CardId {
    /// Returns the raw id as a usize
    pub fn raw(&self) -> usize {
        self.0
    }

    pub fn from_raw(raw: usize) -> Self {
        Self(raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, EnumIter,
)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Joker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, EnumCount)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}
