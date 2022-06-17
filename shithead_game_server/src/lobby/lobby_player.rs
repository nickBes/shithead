use serde::Serialize;

use crate::cards::{Card, CardId, CardsDeck, Rank, CARDS_BY_ID};

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

    /// Returns an iterator over the ids of the player's cards which can be placed on the given card rank.
    pub fn what_cards_can_be_placed_on(&self, on_rank: Rank) -> CardsWhichCanBePlacedOn {
        if !self.cards_in_hand.is_empty() {
            // if the player has some cards in his hand, he can only use them
            CardsWhichCanBePlacedOn::VisibleCards {
                usable_cards: self.cards_in_hand.iter(),
                on_rank,
            }
        } else if !self.three_up_cards.is_empty() {
            // if the player has no cards in his hand, but has some up cards, he can only use them
            CardsWhichCanBePlacedOn::VisibleCards {
                usable_cards: self.three_up_cards.iter(),
                on_rank,
            }
        } else {
            // if the player has no cards in his hand and no up cards, he can only use his down
            // cards.
            CardsWhichCanBePlacedOn::DownCards(self.three_down_cards.iter())
        }
    }
}

/// An iterator over the ids of of the player's cards which can be placed on some card rank.
pub enum CardsWhichCanBePlacedOn<'a> {
    /// The user has finished all of his visible cards, so he can place any of his down cards no
    /// matter what the card rank is.
    DownCards(std::slice::Iter<'a, CardId>),

    /// The user still hasn't finished all of his visible cards, so he can only use cards which
    /// obey the game's rules, among his usable cards.
    VisibleCards {
        /// The cards which are currently available for the player to use.
        usable_cards: std::slice::Iter<'a, CardId>,

        /// The card rank which the cards are to be placed on.
        on_rank: Rank,
    },
}

impl<'a> Iterator for CardsWhichCanBePlacedOn<'a> {
    type Item = CardId;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CardsWhichCanBePlacedOn::DownCards(down_cards) => down_cards.next().copied(),
            CardsWhichCanBePlacedOn::VisibleCards {
                usable_cards,
                on_rank,
            } => loop {
                let cur_card_id = *usable_cards.next()?;
                let card = CARDS_BY_ID.get_card(cur_card_id);
                if !card.can_be_placed_on(*on_rank) {
                    continue;
                }
                return Some(cur_card_id);
            },
        }
    }
}
