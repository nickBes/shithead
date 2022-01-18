use dashmap::{DashMap, DashSet};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::game_server::ClientId;

pub const MAX_PLAYERS_IN_LOBBY: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CardId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub rank: (),
    pub suit: (),
}

/// A cache of all cards by their id
struct CardsById {
    cards_by_id: Vec<Card>,
}
impl CardsById {
    /// Initializes a cache of all cards by their id.
    pub fn new() -> Self {
        let mut cards_by_id = Vec::new();

        // this part is just for testing purposes
        // in the future when the card struct is fully implemented this will initialize an actual deck
        for _ in 0..54 {
            cards_by_id.push(Card { rank: (), suit: () })
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
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(transparent)]
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
    Started,
}

/// A game lobby
#[derive(Debug, Serialize)]
pub struct Lobby {
    name: String,
    state: LobbyState,
    deck: DashSet<CardId>,
    owner: ClientId,
    players: DashMap<ClientId, LobbyPlayer>,
}

impl Lobby {
    /// Creates a new lobby with the given name and owner.
    pub fn new(name: String, owner_id: ClientId) -> Self {
        let deck = DashSet::new();
        for index in 0..CARDS_BY_ID.cards_amount() {
            deck.insert(CardId(index));
        }

        let players = DashMap::new();
        players.insert(owner_id, LobbyPlayer::without_any_cards());

        Self {
            state: LobbyState::Waiting,
            owner: owner_id,
            name,
            deck,
            players,
        }
    }

    /// The amount of players in the lobby.
    pub fn players_amount(&self) -> usize {
        self.players.len()
    }

    /// The current state of the lobby.
    pub fn state(&self) -> LobbyState {
        self.state
    }

    /// The name of the lobby.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a player to the lobby without checking performing any checks.
    /// The checks are done in `GameServerState::join_lobby`.
    ///
    /// The player starts with no cards at all, since assuming checks have been done, the lobby
    /// should be in the `LobbyState::Waiting` state, in which no players have cards.
    pub fn add_player(&self, player_id: ClientId) {
        self.players.insert(player_id, LobbyPlayer::without_any_cards());
    }

    /// Removes the player with the given id from the lobby, and moves make another player the
    /// owner.
    pub fn remove_player(&mut self, player_id: ClientId) -> RemovePlayerFromLobbyResult {
        self.players.remove(&player_id);

        // if the removed player was the owner
        if player_id == self.owner {
            match self.players.iter().next() {
                None => {
                    // if there are no players left
                    RemovePlayerFromLobbyResult::LobbyNowEmpty
                }
                Some(new_owner_entry) => {
                    let new_owner_id = *new_owner_entry.key();
                    self.owner = new_owner_id;
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
}
