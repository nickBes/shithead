use crate::game_server::GAME_SERVER_STATE;
use std::{
    collections::{HashMap, HashSet},
    ops::Index,
    sync::Arc,
    time::Duration,
};

use lazy_static::lazy_static;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, IntoEnumIterator};
use tokio::sync::{broadcast, Notify};
use typescript_type_def::TypeDef;

use crate::{
    card::{Card, CardId, Rank, Suit},
    game_server::{ClientId, BROADCAST_CHANNEL_CAPACITY},
    messages::ServerMessage,
};

pub const MAX_PLAYERS_IN_LOBBY: usize = 6;
const JOKERS_AMOUNT: usize = 2;

pub const INITIAL_CARDS_IN_HAND_AMOUNT: usize = 3;
pub const INITIAL_THREE_CARDS_UP_AMOUNT: usize = 3;
pub const INITIAL_THREE_CARDS_DOWN_AMOUNT: usize = 3;

pub const TURN_DURATION: Duration = Duration::from_secs(3);

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
        self.cards_by_id[card_id.raw()]
    }
}

lazy_static! {
    static ref CARDS_BY_ID: CardsById = CardsById::new();
    static ref ALL_CARD_IDS: Vec<CardId> = (0..CARDS_BY_ID.cards_amount())
        .map(CardId::from_raw)
        .collect();
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, TypeDef)]
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

/// A timer which waits for the next turn, and once the time is out, switches the turn.
#[derive(Debug)]
pub struct NextTurnTimer {
    task: tokio::task::JoinHandle<()>,
    notify: Arc<Notify>,
}
impl NextTurnTimer {
    pub async fn stop(self) {
        self.notify.notify_one();
        self.task.await.unwrap();
    }
}

/// Represents a turn of a player in the game lobby.
#[derive(Debug)]
pub struct Turn {
    player_id: ClientId,
    next_turn_timer: NextTurnTimer,
}
impl Turn {
    pub fn new(lobby_id: LobbyId, player_id: ClientId) -> Self {
        let notify = Arc::new(Notify::new());
        let notify_clone = Arc::clone(&notify);
        let next_turn_timer = NextTurnTimer {
            task: tokio::spawn(async move {
                match tokio::time::timeout(TURN_DURATION, notify_clone.notified()).await {
                    Ok(()) => {
                        // we got a notification, which means the client played its turn before the
                        // time was up, so we can just stop the next turn timer.
                        return;
                    }
                    Err(_) => {
                        // if a timeout has occured, we must switch to the next turn
                        GAME_SERVER_STATE.turn_timeout(lobby_id);
                    }
                }
            }),
            notify,
        };
        Self {
            player_id,
            next_turn_timer,
        }
    }

    /// Stops the next turn timer.
    async fn stop_next_turn_timer(self) {
        self.next_turn_timer.stop().await
    }
}

/// A list of players in the lobby, ordered by their turns.
#[derive(Debug)]
struct OrderedLobbyPlayers {
    players_by_id: HashMap<ClientId, LobbyPlayer>,
    turns_order: Vec<ClientId>,
}

impl OrderedLobbyPlayers {
    fn new() -> Self {
        Self {
            players_by_id: HashMap::new(),
            turns_order: Vec::new(),
        }
    }

    fn insert(&mut self, player_id: ClientId, lobby_player: LobbyPlayer) {
        self.players_by_id.insert(player_id, lobby_player);
        self.turns_order.push(player_id);
    }

    fn remove(&mut self, player_id: ClientId) -> Option<LobbyPlayer> {
        let result = self.players_by_id.remove(&player_id);
        if result.is_some() {
            // if there was a player with this id, remove this player from the turns order vector as well
            self.turns_order
                .retain(|&player_id_to_check| player_id_to_check != player_id);
        }
        result
    }

    fn len(&self) -> usize {
        self.turns_order.len()
    }

    fn is_empty(&self) -> bool {
        self.turns_order.is_empty()
    }

    fn player_ids<'a>(&'a self) -> impl Iterator<Item = ClientId> + 'a {
        self.turns_order.iter().copied()
    }

    fn values_mut<'a>(
        &'a mut self,
    ) -> std::collections::hash_map::ValuesMut<'a, ClientId, LobbyPlayer> {
        self.players_by_id.values_mut()
    }

    /// Finds the id of the player that will play after the player with the given id, according to
    /// the turns order.
    fn find_next_turn_player_after(&self, player_id: ClientId) -> ClientId {
        // the index of the given player in the turns order vector
        let given_player_index = self
            .turns_order
            .iter()
            .position(|&player_id_to_check| player_id_to_check == player_id)
            .expect("attempted to find the next turn after a player which is not in the game");

        // find the index in the turns order vector of the next turn player
        let next_index = (given_player_index + 1) % self.turns_order.len();

        // return the id of the next turn player
        self.turns_order[next_index]
    }

    /// Returns the player which is the first one to play.
    fn first_turn_player(&self) -> ClientId {
        self.turns_order[0]
    }

    /// Gives cards to a player.
    fn give_cards_to_player(&mut self, player_id: ClientId, cards: impl Iterator<Item = CardId>) {
        if let Some(player) = self.players_by_id.get_mut(&player_id) {
            player.cards_in_hand.extend(cards);
        }
    }
}

impl Index<ClientId> for OrderedLobbyPlayers {
    type Output = LobbyPlayer;

    fn index(&self, index: ClientId) -> &Self::Output {
        &self.players_by_id[&index]
    }
}

/// A game lobby
#[derive(Debug)]
pub struct Lobby {
    id: LobbyId,
    name: String,
    state: LobbyState,
    deck: CardsDeck,
    trash: CardsDeck,
    player_list: OrderedLobbyPlayers,
    owner_id: ClientId,
    current_turn: Option<Turn>,
    pub broadcast_messages_sender: broadcast::Sender<ServerMessage>,
}

impl Lobby {
    /// Creates a new lobby with the given name and owner.
    pub fn new(id: LobbyId, name: String, owner_id: ClientId) -> Self {
        let mut players = OrderedLobbyPlayers::new();
        players.insert(owner_id, LobbyPlayer::without_any_cards());

        let (broadcast_messages_sender, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        Self {
            state: LobbyState::Waiting,
            deck: CardsDeck::shuffled(),
            trash: CardsDeck::empty(),
            current_turn: None,
            owner_id,
            id,
            name,
            player_list: players,
            broadcast_messages_sender,
        }
    }

    /// The amount of players in the lobby.
    pub fn players_amount(&self) -> usize {
        self.player_list.len()
    }

    /// Is the lobby empty?
    pub fn is_empty(&self) -> bool {
        self.player_list.is_empty()
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
        self.player_list.player_ids()
    }

    /// Adds a player to the lobby without performing any checks.
    /// The checks are done in `GameServerState::join_lobby`.
    ///
    /// The player starts with no cards at all, since assuming checks have been done, the lobby
    /// should be in the `LobbyState::Waiting` state, in which no players have cards.
    pub fn add_player(&mut self, player_id: ClientId) {
        self.player_list
            .insert(player_id, LobbyPlayer::without_any_cards());
    }

    /// Removes the player with the given id from the lobby, and moves make another player the
    /// owner.
    pub fn remove_player(&mut self, player_id: ClientId) -> RemovePlayerFromLobbyResult {
        if self.player_list.remove(player_id).is_none() {
            return RemovePlayerFromLobbyResult::PlayerWasntInLobby;
        }

        // if the removed player was the owner
        if player_id == self.owner_id {
            match self.player_list.player_ids().next() {
                None => {
                    // if there are no players left
                    RemovePlayerFromLobbyResult::LobbyNowEmpty
                }
                Some(new_owner_id) => {
                    self.owner_id = new_owner_id;
                    RemovePlayerFromLobbyResult::NewOwner(new_owner_id)
                }
            }
        } else {
            if self.player_list.is_empty() {
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

        for player in self.player_list.values_mut() {
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

        // start the first turn
        let first_turn_player_id = self.player_list.first_turn_player();
        self.set_current_turn_and_update_players(first_turn_player_id);
    }

    /// Returns the lobby player with the given id.
    pub fn get_player(&self, player_id: ClientId) -> &LobbyPlayer {
        &self.player_list[player_id]
    }

    /// Finds the id of the player who will play in the next turn.
    fn next_turn_player_id(&self) -> ClientId {
        match &self.current_turn {
            Some(current_turn) => self
                .player_list
                .find_next_turn_player_after(current_turn.player_id),
            None => self.player_list.first_turn_player(),
        }
    }

    /// The player has finished playing his turn, move on the next turn.
    pub async fn turn_finished(&mut self) {
        let new_turn_player_id = self.next_turn_player_id();

        // if there is still a timer running for the current turn, cancel it.
        if let Some(current_turn) = std::mem::take(&mut self.current_turn) {
            current_turn.stop_next_turn_timer().await
        }

        self.set_current_turn_and_update_players(new_turn_player_id);
    }

    /// Switches to the next turn.
    pub fn turn_timeout(&mut self) {
        let new_turn_player_id = self.next_turn_player_id();

        let current_turn = std::mem::take(&mut self.current_turn)
            .expect("turn timeout called but there is no current turn");

        // give the player all the cards in the trash
        self.player_list
            .give_cards_to_player(current_turn.player_id, self.trash.take_all());

        self.set_current_turn_and_update_players(new_turn_player_id);
    }

    /// Sets the current turn to the player with the given id and updates the players about the new
    /// turn.
    fn set_current_turn_and_update_players(&mut self, new_turn_player_id: ClientId) {
        self.current_turn = Some(Turn::new(self.id, new_turn_player_id));

        // TODO: update all players about this turn.
    }
}

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

/// The result of removing a player from a lobby.
pub enum RemovePlayerFromLobbyResult {
    Ok,
    NewOwner(ClientId),
    LobbyNowEmpty,
    PlayerWasntInLobby,
}
