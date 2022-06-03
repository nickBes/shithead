use crate::cards::Card;
use crate::cards::CardId;
use crate::cards::Rank;
use crate::cards::Suit;
use crate::cards::JOKERS_AMOUNT;
use strum::EnumCount;
use strum::IntoEnumIterator;

use lazy_static::lazy_static;

/// A cache of all cards by their id
pub struct CardsById {
    cards_by_id: Vec<Card>,
}
impl CardsById {
    /// Initializes a cache of all cards by their id.
    pub fn new() -> Self {
        let mut cards_by_id = Vec::new();

        // initialize with all possible cards
        for rank in Rank::iter() {
            // if the card is a joker use only 2 suits, otherwise use all suits
            let suits_amount = if rank == Rank::Joker {
                JOKERS_AMOUNT
            } else {
                Suit::COUNT
            };
            for suit in Suit::iter().take(suits_amount) {
                cards_by_id.push(Card { rank, suit })
            }
        }

        Self { cards_by_id }
    }

    /// The total amount of cards in a single deck
    pub fn cards_amount(&self) -> usize {
        self.cards_by_id.len()
    }

    /// Returns a card given its id
    pub fn get_card(&self, card_id: CardId) -> Card {
        self.cards_by_id[card_id.raw()]
    }
}

lazy_static! {
    pub static ref CARDS_BY_ID: CardsById = CardsById::new();
    pub static ref ALL_CARD_IDS: Vec<CardId> = (0..CARDS_BY_ID.cards_amount())
        .map(CardId::from_raw)
        .collect();
}
