mod lobby_player;
mod ordered_lobby_players;
mod in_game;
mod turn;

use crate::{
    cards::{CardsDeck, Rank, CARDS_BY_ID},
    game_server::{ExposedLobbyPlayerInfo, GameServerError, GAME_SERVER_STATE},
    messages::ClickedCardLocation,
    some_or_return,
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

use self::{lobby_player::LobbyPlayer, ordered_lobby_players::OrderedLobbyPlayers, turn::Turn, in_game::{InGameLobbyState, InGameLobbyStateMut}};

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
impl std::fmt::Display for LobbyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A state of a lobby.
#[derive(Debug)]
pub enum LobbyState {
    Waiting,
    ChoosingTop3,
    GameStarted(InGameLobbyState),
}

/// Lobby data that is not state dependent.
#[derive(Debug)]
pub struct LobbyNonStateData {
    id: LobbyId,
    name: String,
    player_list: OrderedLobbyPlayers,
    owner_id: ClientId,
    pub broadcast_messages_sender: broadcast::Sender<ServerMessage>,
}

/// A game lobby
#[derive(Debug)]
pub struct Lobby {
    state: LobbyState,
    non_state_data: LobbyNonStateData,
}

impl Lobby {
    /// Creates a new lobby with the given name and owner.
    pub fn new(id: LobbyId, name: String, owner_id: ClientId) -> Self {
        let mut players = OrderedLobbyPlayers::new();
        players.insert(owner_id, LobbyPlayer::without_any_cards());

        let (broadcast_messages_sender, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        Self {
            state: LobbyState::Waiting,
            non_state_data: LobbyNonStateData {
                owner_id,
                id,
                name,
                player_list: players,
                broadcast_messages_sender,
            },
        }
    }

    /// Returns a reference to the lobby's broadcast messages sender.
    pub fn broadcast_messages_sender(&self) -> &broadcast::Sender<ServerMessage> {
        &self.non_state_data.broadcast_messages_sender
    }

    /// The id of the lobby
    pub fn id(&self) -> LobbyId {
        self.non_state_data.id
    }

    /// The amount of players in the lobby.
    pub fn players_amount(&self) -> usize {
        self.non_state_data.player_list.len()
    }

    /// Is the lobby empty?
    pub fn is_empty(&self) -> bool {
        self.non_state_data.player_list.is_empty()
    }

    /// The current state of the lobby.
    pub fn state(&self) -> &LobbyState {
        &self.state
    }

    /// Is the lobby in a waiting state?
    pub fn is_waiting(&self) -> bool {
        matches!(&self.state, LobbyState::Waiting)
    }

    /// The name of the lobby.
    pub fn name(&self) -> &str {
        &self.non_state_data.name
    }

    /// The id of the owner.
    pub fn owner_id(&self) -> ClientId {
        self.non_state_data.owner_id
    }

    /// The ids of the players in the lobby.
    pub fn player_ids<'a>(&'a self) -> impl Iterator<Item = ClientId> + 'a {
        self.non_state_data.player_list.player_ids()
    }

    /// Returns a list of exposed information about each player.
    pub fn exposed_player_list(&self) -> Vec<ExposedLobbyPlayerInfo> {
        self.player_ids().map(ExposedLobbyPlayerInfo::new).collect()
    }

    /// Returns a reference to the in game state of this lobby, if it is in an in game state.
    fn in_game_state(&self) -> Option<&InGameLobbyState> {
        match &self.state {
            LobbyState::GameStarted(state) => Some(state),
            _ => None,
        }
    }

    /// Returns a mutable reference to the in game state of this lobby, if it is in an in game state.
    fn in_game_state_mut(&mut self) -> Option<InGameLobbyStateMut> {
        match &mut self.state {
            LobbyState::GameStarted(state) => Some(InGameLobbyStateMut {
                in_game_state: state,
                lobby_data: &mut self.non_state_data,
            }),
            _ => None,
        }
    }

    /// Adds a player to the lobby without performing any checks.
    /// The checks are done in `GameServerState::join_lobby`.
    ///
    /// The player starts with no cards at all, since assuming checks have been done, the lobby
    /// should be in the `LobbyState::Waiting` state, in which no players have cards.
    pub fn add_player(&mut self, player_id: ClientId) {
        self.non_state_data
            .player_list
            .insert(player_id, LobbyPlayer::without_any_cards());
    }

    /// Removes the player with the given id from the lobby.
    /// If that player was the current turn, moves the current turn to the next player.
    /// If that player was the owner, makes another player the owner.
    pub async fn remove_player(&mut self, player_id: ClientId) -> RemovePlayerFromLobbyResult {
        // if the removed player was the current turn, move the turn to the next player.
        //
        // this must be done before removing the player, because if we first removed it there would
        // be no way to find who's the next player after him.
        if let Some(in_game_state) = self.in_game_state_mut() {
            if in_game_state.current_turn_player_id() == player_id {
                self.turn_finished().await
            }
        }

        // remove the player
        if self.non_state_data.player_list.remove(player_id).is_none() {
            return RemovePlayerFromLobbyResult::PlayerWasntInLobby;
        }

        // if the removed player was the owner
        if player_id == self.owner_id() {
            match self.non_state_data.player_list.player_ids().next() {
                None => {
                    // if there are no players left
                    RemovePlayerFromLobbyResult::LobbyNowEmpty
                }
                Some(new_owner_id) => {
                    self.non_state_data.owner_id = new_owner_id;
                    RemovePlayerFromLobbyResult::NewOwner(new_owner_id)
                }
            }
        } else {
            if self.is_empty() {
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
        let mut deck = CardsDeck::shuffled();

        for player in self.non_state_data.player_list.players_mut() {
            player.cards_in_hand = deck
                .take_cards_from_top(INITIAL_CARDS_IN_HAND_AMOUNT)
                .expect("not enough cards to initialize game")
                .collect();
            player.three_up_cards = deck
                .take_cards_from_top(INITIAL_THREE_CARDS_UP_AMOUNT)
                .expect("not enough cards to initialize game")
                .collect();
            player.three_up_cards = deck
                .take_cards_from_top(INITIAL_THREE_CARDS_DOWN_AMOUNT)
                .expect("not enough cards to initialize game")
                .collect();
        }

        // start the first turn
        let first_turn_player_id = self.non_state_data.player_list.first_turn_player();
        let first_turn = Turn::new(self.id(), first_turn_player_id);

        // update all the players about this turn.
        let _ = self
            .broadcast_messages_sender()
            .send(ServerMessage::Turn(first_turn_player_id));

        // set the state of the lobby to an in game state.
        self.state = LobbyState::GameStarted(InGameLobbyState {
            deck,
            trash: CardsDeck::empty(),
            current_turn: first_turn,
        })
    }

    /// Returns the lobby player with the given id.
    pub fn get_player(&self, player_id: ClientId) -> Option<&LobbyPlayer> {
        self.non_state_data.player_list.get_player(player_id)
    }

    /// The player has finished playing his turn, move on the next turn.
    pub async fn turn_finished(&mut self) {
        let mut in_game_state = some_or_return!(self.in_game_state_mut());

        in_game_state.advance_turn_and_update_players().await;
    }

    /// The current turn has timed out, give the current turn player the trash or play one of his
    /// cards, and advance to the next turn.
    pub async fn turn_timeout(&mut self) {
        let mut in_game_state = some_or_return!(self.in_game_state_mut());

        in_game_state.turn_timeout().await;
    }

    /// Handles a card click from one of the clients.
    pub async fn click_card(
        &mut self,
        client_id: ClientId,
        clicked_card_location: ClickedCardLocation,
    ) -> Result<(), GameServerError> {
        let mut in_game_state = self
            .in_game_state_mut()
            .ok_or(GameServerError::GameHasntStartedYet)?;

        in_game_state
            .click_card(client_id, clicked_card_location)
            .await
    }
}

/// The result of removing a player from a lobby.
pub enum RemovePlayerFromLobbyResult {
    Ok,
    NewOwner(ClientId),
    LobbyNowEmpty,
    PlayerWasntInLobby,
}
