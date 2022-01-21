use rand::prelude::SliceRandom;
use crate::cards::cards_by_id_cache::ALL_CARD_IDS;
use crate::cards::CardId;

/// A deck of cards
#[derive(Debug)]
pub struct CardsDeck {
    cards: Vec<CardId>,
}
impl CardsDeck {
    /// Creates a shuffled deck of cards.
    pub fn shuffled() -> Self {
        // first collect into a vector so that we can shuffle it.
        let mut card_ids = ALL_CARD_IDS.clone();
        card_ids.shuffle(&mut rand::thread_rng());

        // after we have a shuffled vector of card ids, we can collect it into a DashSet to
        // represent our deck.
        let cards: Vec<CardId> = card_ids.iter().copied().collect();

        Self { cards }
    }

    /// Creates an empty deck of cards.
    pub fn empty() -> Self {
        Self { cards: Vec::new() }
    }

    /// Takes cards from the top of the deck.
    pub fn take_cards_from_top<'a>(
        &'a mut self,
        amount: usize,
    ) -> Option<impl Iterator<Item = CardId> + 'a> {
        if amount > self.cards.len() {
            return None;
        }
        Some(self.cards.drain(self.cards.len() - amount..))
    }

    /// Takes all the cards from the deck.
    pub fn take_all<'a>(&'a mut self) -> std::vec::Drain<'a, CardId> {
        self.cards.drain(..)
    }
}

