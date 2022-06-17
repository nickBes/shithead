use crate::cards::cards_by_id_cache::ALL_CARD_IDS;
use crate::cards::CardId;
use rand::prelude::SliceRandom;

/// A deck of cards
#[derive(Debug)]
pub struct CardsDeck {
    /// The cards in the deck, ordered from bottom to top, such that the card at index `0` is at the
    /// bottom of the deck, and the card at index `cards.len() - 1` is the card at the top of the
    /// deck.
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

    /// Returns the card at the top of this deck
    pub fn top_card(&self)->Option<CardId>{
        self.cards.last().copied()
    }

    /// The cards in the deck, ordered from bottom to top, such that the card at index `0` is at the
    /// bottom of the deck, and the card at index `length - 1` is the card at the top of the
    /// deck.
    pub fn cards_bottom_to_top(&self)->&[CardId]{
        &self.cards
    }

    /// Takes all the cards from the deck.
    pub fn take_all<'a>(&'a mut self) -> std::vec::Drain<'a, CardId> {
        self.cards.drain(..)
    }
}

