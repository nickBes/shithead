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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TypeDef)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    /// Can this card be placed on a card with the given rank?
    pub fn can_be_placed_on(self, on_rank: Rank) -> bool {
        match (self.rank, on_rank){
            (_, Rank::Two) => true,
            (_, Rank::Three)=> panic!("can't check if card can be placed on a 3, the card below the 3 should be compared instead"),
            (rank_to_place, Rank::Seven)=> rank_to_place <= Rank::Seven,
            (_, Rank::Ten) => panic!("cards can't be placed on a 10, since it burns the entire trash, and will never be the top of the trash"),
            (_, Rank::Joker) =>panic!("cards can't be placed on a joker, since it clears the entire trash, and will never be the top of the trash"),
            (rank_to_place, on_rank) => rank_to_place >= on_rank,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, EnumIter, TypeDef
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, EnumCount, TypeDef)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}
