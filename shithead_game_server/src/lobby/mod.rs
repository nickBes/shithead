mod lobby_player;
mod ordered_lobby_players;
mod turn;

use crate::{
    cards::{CardsDeck, Rank, CARDS_BY_ID},
    game_server::{GameServerError, GAME_SERVER_STATE},
    messages::ClickedCardLocation,
};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use typescript_type_def::TypeDef;

use crate::{
    cards::CardId,
    game_server::{ClientId, BROADCAST_CHANNEL_CAPACITY},
    messages::ServerMessage,
};

use self::{lobby_player::LobbyPlayer, ordered_lobby_players::OrderedLobbyPlayers, turn::Turn};

pub const MAX_PLAYERS_IN_LOBBY: usize = 6;

pub const INITIAL_CARDS_IN_HAND_AMOUNT: usize = 3;
pub const INITIAL_THREE_CARDS_UP_AMOUNT: usize = 3;
pub const INITIAL_THREE_CARDS_DOWN_AMOUNT: usize = 3;

pub const TURN_DURATION: Duration = Duration::from_secs(3);

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

        for player in self.player_list.players() {
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
    pub fn get_player(&self, player_id: ClientId) -> Option<&LobbyPlayer> {
        self.player_list.get_player(player_id)
    }

    /// Finds the id of the player who will play in the next turn.
    fn next_turn_player_id(&self) -> ClientId {
        let current_turn = self
            .current_turn
            .as_ref()
            .expect("requesting the next turn while there is no current turn");
        self.player_list
            .find_next_turn_player_after(current_turn.player_id())
    }

    /// The player has finished playing his turn, move on the next turn.
    // this is currently unused but will be used once the `ClickedCard` message is implemented.
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
            .give_cards_to_player(current_turn.player_id(), self.trash.take_all());

        // let all the players in the lobby know that the player who timed out got all cards from
        // the trash.
        let _ = self
            .broadcast_messages_sender
            .send(ServerMessage::GiveTrash(current_turn.player_id()));

        self.set_current_turn_and_update_players(new_turn_player_id);
    }

    /// Sets the current turn to the player with the given id and updates the players about the new
    /// turn.
    fn set_current_turn_and_update_players(&mut self, new_turn_player_id: ClientId) {
        self.current_turn = Some(Turn::new(self.id, new_turn_player_id));

        // update all the players about this turn.
        let _ = self
            .broadcast_messages_sender
            .send(ServerMessage::Turn(new_turn_player_id));
    }

    /// The value of the trash's top card.
    /// This is used to check if a card can be placed on the trash.
    /// If the trash's top card is a three, it returns value of the card below it.
    /// Otherwise just returns the value of the trash's top card.
    /// If the trash is empty, returns a rank of 2, to indicate that any card can be placed in the
    /// trash.
    fn trash_top_card_rank(&self) -> Rank {
        let trash_cards_bottom_to_top = self.trash.cards_bottom_to_top();
        let cards_top_to_bottom = trash_cards_bottom_to_top.iter().rev();
        for &card_id in cards_top_to_bottom {
            let card = CARDS_BY_ID.get_card(card_id);
            if card.rank != Rank::Three {
                return card.rank;
            }
        }

        // if the deck is empty or only has 3's in it, the value of it is a 2, which
        // indicates that any card can be placed in it.
        Rank::Two
    }

    /// Handles a card click from one of the clients.
    pub async fn click_card(
        &mut self,
        client_id: ClientId,
        clicked_card_location: ClickedCardLocation,
    ) -> Result<(), GameServerError> {
        let current_turn_player_id = self
            .current_turn
            .as_ref()
            .ok_or(GameServerError::GameHasntStartedYet)?
            .player_id();

        if current_turn_player_id != client_id {
            return Err(GameServerError::NotYourTurn);
        }

        let player = self
            .player_list
            .get_player(client_id)
            .ok_or(GameServerError::NotInALobby)?;

        match clicked_card_location {
            ClickedCardLocation::Trash => {
                // if the player can place any of his cards in the trash, then he is not allowed to
                // take the trash
                if player.what_cards_can_be_placed_on(self.trash_top_card_rank()).next().is_some(){
                    return Err(GameServerError::CantTakeTrashBecauseSomeCardsCanBePlayed)
                }

                // the player has finished his turn
                self.turn_finished().await;

                // give the player all the cards in the trash
                self.player_list
                    .give_cards_to_player(current_turn_player_id, self.trash.take_all());

                // let all the players in the lobby know that the player who timed out got all cards from
                // the trash.
                let _ = self
                    .broadcast_messages_sender
                    .send(ServerMessage::GiveTrash(current_turn_player_id));

                Ok(())
            }
            ClickedCardLocation::MyCards { card_index } => {
                todo!()
            }
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
