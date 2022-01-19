use std::collections::HashMap;

use lazy_static::lazy_static;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, IntoEnumIterator};
use tokio::sync::broadcast;

use crate::{
    card::{Card, Rank, Suit},
    game_server::{ClientId, BROADCAST_CHANNEL_CAPACITY},
    messages::ServerMessage,
};

pub const MAX_PLAYERS_IN_LOBBY: usize = 6;
const JOKERS_AMOUNT: usize = 2;

pub const INITIAL_CARDS_IN_HAND_AMOUNT: usize = 3;
pub const INITIAL_THREE_CARDS_UP_AMOUNT: usize = 3;
pub const INITIAL_THREE_CARDS_DOWN_AMOUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CardId(usize);

/// A cache of all cards by their id
struct CardsById {
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
        self.cards_by_id[card_id.0]
    }
}

lazy_static! {
    static ref CARDS_BY_ID: CardsById = CardsById::new();
    static ref ALL_CARD_IDS: Vec<CardId> = (0..CARDS_BY_ID.cards_amount()).map(CardId).collect();
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LobbyId(usize);
impl LobbyId {
    /// Creates a LobbyId from a raw id. Only call this on valid lobby ids created by getting the
    /// next lobby id from the server's state.
    pub fn from_raw(raw: usize) -> Self {
        Self(raw)
    }
}

/// A state of a lobby.
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy, Hash)]
pub enum LobbyState {
    Waiting,
    GameStarted,
}

/// A deck of cards
#[derive(Debug)]
pub struct Deck {
    cards: Vec<CardId>,
}
impl Deck {
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
}

/// A game lobby
#[derive(Debug)]
pub struct Lobby {
    name: String,
    state: LobbyState,
    deck: Deck,
    owner_id: ClientId,
    players: HashMap<ClientId, LobbyPlayer>,
    pub broadcast_messages_sender: broadcast::Sender<ServerMessage>,
}

impl Lobby {
    /// Creates a new lobby with the given name and owner.
    pub fn new(name: String, owner_id: ClientId) -> Self {
        let mut players = HashMap::new();
        players.insert(owner_id, LobbyPlayer::without_any_cards());

        let (broadcast_messages_sender, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        Self {
            state: LobbyState::Waiting,
            deck: Deck::shuffled(),
            owner_id,
            name,
            players,
            broadcast_messages_sender,
        }
    }

    /// The amount of players in the lobby.
    pub fn players_amount(&self) -> usize {
        self.players.len()
    }

    /// Is the lobby empty?
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    /// The current state of the lobby.
    pub fn state(&self) -> LobbyState {
        self.state
    }

    /// The name of the lobby.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The id of the owner.
    pub fn owner_id(&self) -> ClientId {
        self.owner_id
    }

    /// The ids of the players in the lobby.
    pub fn player_ids<'a>(&'a self) -> impl Iterator<Item = ClientId> + 'a {
        self.players.keys().copied()
    }

    /// Adds a player to the lobby without performing any checks.
    /// The checks are done in `GameServerState::join_lobby`.
    ///
    /// The player starts with no cards at all, since assuming checks have been done, the lobby
    /// should be in the `LobbyState::Waiting` state, in which no players have cards.
    pub fn add_player(&mut self, player_id: ClientId) {
        self.players
            .insert(player_id, LobbyPlayer::without_any_cards());
    }

    /// Removes the player with the given id from the lobby, and moves make another player the
    /// owner.
    pub fn remove_player(&mut self, player_id: ClientId) -> RemovePlayerFromLobbyResult {
        if self.players.remove(&player_id).is_none() {
            return RemovePlayerFromLobbyResult::PlayerWasntInLobby;
        }

        // if the removed player was the owner
        if player_id == self.owner_id {
            match self.players.keys().next() {
                None => {
                    // if there are no players left
                    RemovePlayerFromLobbyResult::LobbyNowEmpty
                }
                Some(&new_owner_id) => {
                    self.owner_id = new_owner_id;
                    RemovePlayerFromLobbyResult::NewOwner(new_owner_id)
                }
            }
        } else {
            if self.players.is_empty() {
                // the lobby is empty
                RemovePlayerFromLobbyResult::LobbyNowEmpty
            } else {
                // the lobby is not empty and the owner hasn't changed
                RemovePlayerFromLobbyResult::Ok
            }
        }
    }

    /// Starts the game in this lobby without performing any checks.
    /// The checks are done in `GameServerState::start_game`.
    ///
    /// This function gives all players the initial amount of cards.
    pub fn start_game(&mut self) {
        self.state = LobbyState::GameStarted;

        for player in self.players.values_mut() {
            player.cards_in_hand = self
                .deck
                .take_cards_from_top(INITIAL_CARDS_IN_HAND_AMOUNT)
                .expect("not enough cards to initialize game")
                .collect();
            player.three_up_cards = self
                .deck
                .take_cards_from_top(INITIAL_THREE_CARDS_UP_AMOUNT)
                .expect("not enough cards to initialize game")
                .collect();
            player.three_up_cards = self
                .deck
                .take_cards_from_top(INITIAL_THREE_CARDS_DOWN_AMOUNT)
                .expect("not enough cards to initialize game")
                .collect();
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LobbyPlayer {
    cards_in_hand: Vec<CardId>,
    three_up_cards: Vec<CardId>,
    three_down_cards: Vec<CardId>,
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

/// The result of removing a player from a lobby.
pub enum RemovePlayerFromLobbyResult {
    Ok,
    NewOwner(ClientId),
    LobbyNowEmpty,
    PlayerWasntInLobby,
}
