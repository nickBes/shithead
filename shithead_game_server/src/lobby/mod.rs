mod in_game;
mod lobby_player;
mod ordered_lobby_players;
mod turn;

use crate::{
    cards::{CardsDeck, Rank, CARDS_BY_ID},
    game_server::{ExposedLobbyPlayerInfo, GameServerError, GAME_SERVER_STATE},
    messages::ClickedCardLocation,
    some_or_return,
};

use std::{collections::HashMap, sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Notify};
use typescript_type_def::TypeDef;

use crate::{
    cards::CardId,
    game_server::{ClientId, BROADCAST_CHANNEL_CAPACITY},
    messages::ServerMessage,
};

use self::{
    in_game::{InGameLobbyContext, InGameLobbyState},
    lobby_player::LobbyPlayer,
    ordered_lobby_players::OrderedLobbyPlayers,
    turn::Turn,
};

pub const MAX_PLAYERS_IN_LOBBY: usize = 6;

pub const INITIAL_CARDS_IN_HAND_AMOUNT: usize = 6;

pub const TURN_DURATION: Duration = Duration::from_secs(20);

pub const CHOOSE_TOP_3_DURATION: Duration = Duration::from_secs(20);

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
    Choosing3Up(Choosing3UpState),
    GameStarted(InGameLobbyState),
}

/// A timer for the choosing 3 up lobby state.
#[derive(Debug)]
pub struct Choosing3UpState {
    _timer: Choosing3UpTimer,
    deck: CardsDeck,
}

/// A timer for the choosing 3 up lobby state.
#[derive(Debug)]
pub struct Choosing3UpTimer {
    task: tokio::task::JoinHandle<()>,
    notify: Arc<Notify>,
}
impl Choosing3UpTimer {
    /// Creates a new timer.
    pub fn new(lobby_id: LobbyId) -> Self {
        let notify = Arc::new(Notify::new());
        let notify_clone = Arc::clone(&notify);
        Self {
            task: tokio::spawn(async move {
                match tokio::time::timeout(CHOOSE_TOP_3_DURATION, notify_clone.notified()).await {
                    Ok(()) => {
                        // we got a notification, which means that all client chose their 3 up
                        // cards before the time was up, so we can just stop the timer.
                        return;
                    }
                    Err(_) => {
                        // if a timeout has occured, let the lobby know
                        GAME_SERVER_STATE.choose_3_up_timeout(lobby_id);
                    }
                }
            }),
            notify,
        }
    }

    pub async fn stop(self) {
        // notifying this `Notify` object will notify the task because the task is waiting to
        // receive a notification on it, and when the task will receive the notification it will
        // stop.
        self.notify.notify_one();
        self.task.await.unwrap();
    }
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
    data: LobbyNonStateData,
}

impl Lobby {
    /// Creates a new lobby with the given name and owner.
    pub fn new(id: LobbyId, name: String, owner_id: ClientId) -> Self {
        let mut players = OrderedLobbyPlayers::new();
        players.insert(owner_id, LobbyPlayer::without_any_cards());

        let (broadcast_messages_sender, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        Self {
            state: LobbyState::Waiting,
            data: LobbyNonStateData {
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
        &self.data.broadcast_messages_sender
    }

    /// The id of the lobby
    pub fn id(&self) -> LobbyId {
        self.data.id
    }

    /// The amount of players in the lobby.
    pub fn players_amount(&self) -> usize {
        self.data.player_list.len()
    }

    /// Is the lobby empty?
    pub fn is_empty(&self) -> bool {
        self.data.player_list.is_empty()
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
        &self.data.name
    }

    /// The id of the owner.
    pub fn owner_id(&self) -> ClientId {
        self.data.owner_id
    }

    /// The ids of the players in the lobby.
    pub fn player_ids<'a>(&'a self) -> impl Iterator<Item = ClientId> + 'a {
        self.data.player_list.player_ids()
    }

    /// Returns a list of exposed information about each player.
    pub fn exposed_player_list(&self) -> Vec<ExposedLobbyPlayerInfo> {
        self.player_ids().map(ExposedLobbyPlayerInfo::new).collect()
    }

    /// Returns an in game lobby context for this lobby, if it is in an in game state.
    fn in_game_context(&mut self) -> Option<InGameLobbyContext> {
        match &mut self.state {
            LobbyState::GameStarted(state) => Some(InGameLobbyContext::new(state, &mut self.data)),
            _ => None,
        }
    }

    /// Adds a player to the lobby without performing any checks.
    /// The checks are done in `GameServerState::join_lobby`.
    ///
    /// The player starts with no cards at all, since assuming checks have been done, the lobby
    /// should be in the `LobbyState::Waiting` state, in which no players have cards.
    pub fn add_player(&mut self, player_id: ClientId) {
        self.data
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
        if let Some(in_game_ctx) = self.in_game_context() {
            if in_game_ctx.current_turn_player_id() == player_id {
                self.turn_finished().await
            }
        }

        // remove the player
        if self.data.player_list.remove(player_id).is_none() {
            return RemovePlayerFromLobbyResult::PlayerWasntInLobby;
        }

        // if there is only 1 player left, stop the game.
        if self.players_amount() == 1 {
            self.stop_game().await;
        }

        // if the removed player was the owner
        if player_id == self.owner_id() {
            match self.data.player_list.player_ids().next() {
                None => {
                    // if there are no players left
                    RemovePlayerFromLobbyResult::LobbyNowEmpty
                }
                Some(new_owner_id) => {
                    self.data.owner_id = new_owner_id;
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

        for player in self.data.player_list.players_mut() {
            player.cards_in_hand = deck
                .take_cards_from_top(INITIAL_CARDS_IN_HAND_AMOUNT)
                .expect("not enough cards to initialize game")
                .collect();
            player.three_down_cards = deck
                .take_cards_from_top(3)
                .expect("not enough cards to initialize game")
                .collect();
        }

        // set the state of the lobby to an in game state.
        self.state = LobbyState::Choosing3Up(Choosing3UpState {
            deck,
            _timer: Choosing3UpTimer::new(self.id()),
        })
    }

    /// Returns the lobby player with the given id.
    pub fn get_player(&self, player_id: ClientId) -> Option<&LobbyPlayer> {
        self.data.player_list.get_player(player_id)
    }

    /// The player has finished playing his turn, move on the next turn.
    pub async fn turn_finished(&mut self) {
        let mut in_game_ctx = some_or_return!(self.in_game_context());

        in_game_ctx.advance_turn_and_update_players().await;
    }

    /// The current turn has timed out, give the current turn player the trash or play one of his
    /// cards, and advance to the next turn.
    pub async fn turn_timeout(&mut self) {
        let mut in_game_ctx = some_or_return!(self.in_game_context());

        in_game_ctx.turn_timeout().await;
    }

    /// The timer for choosing the 3 up cards is over.
    pub fn choose_3_up_timeout(&mut self) {
        let chooosing_3_up_state = match &mut self.state {
            LobbyState::Choosing3Up(state) => state,
            _ => return,
        };

        // the 3 up cards of all clients who didn't choose their 3 up cards, and some cards were
        // randomly chosen for them.
        let mut three_up_cards_of_modified_players = HashMap::new();

        // make sure that all players have chosen their 3 up cards, and if they didn't, randomly
        // choose them.
        for (&player_id, player) in &mut self.data.player_list.players_by_id {
            if player.three_up_cards.len() < 3 {
                // if the player doesn't have 3 up cards, choose a bunch of random cards from his
                // hand until he has enough.
                while player.three_up_cards.len() < 3 {
                    player
                    .three_up_cards
                    .push(player.cards_in_hand.pop().expect(
                        "failed to randomly choose 3 up cards for player because he ran out of cards",
                    ));
                }

                // add this player to the list of modified players
                three_up_cards_of_modified_players.insert(player_id, player.three_up_cards.clone());
            }
        }

        // update all players about the face that the 3 up selection is over and about the modified
        // players.
        let _ = self
            .data
            .broadcast_messages_sender
            .send(ServerMessage::TheeUpSelectionDone {
                three_up_cards_of_modified_players,
            });

        // start the first turn
        let first_turn_player_id = self.data.player_list.first_turn_player();
        let first_turn = Turn::new(self.data.id, first_turn_player_id);

        // update all the players about this turn.
        let _ = self
            .data
            .broadcast_messages_sender
            .send(ServerMessage::Turn(first_turn_player_id));

        // set the state of the lobby to an in game state.
        self.state = LobbyState::GameStarted(InGameLobbyState {
            deck: std::mem::replace(&mut chooosing_3_up_state.deck, CardsDeck::empty()),
            trash: CardsDeck::empty(),
            current_turn: first_turn,
        })
    }

    /// Handles a card click from one of the clients.
    pub async fn click_card(
        &mut self,
        client_id: ClientId,
        clicked_card_location: ClickedCardLocation,
    ) -> Result<(), GameServerError> {
        match &mut self.state {
            LobbyState::Waiting => return Err(GameServerError::GameHasntStartedYet),
            LobbyState::Choosing3Up(_) => {
                let player = self
                    .data
                    .player_list
                    .get_player_mut(client_id)
                    .ok_or(GameServerError::NotInALobby)?;
                match clicked_card_location {
                    ClickedCardLocation::FromCardsInHand { card_index } => {
                        // the player clicked a card from his hand during the 3 up selection, move
                        // that card to his 3 up cards.
                        let card_index = card_index as usize;
                        if card_index >= player.cards_in_hand.len() {
                            return Err(GameServerError::NoSuchCard);
                        }
                        let card = player.cards_in_hand.swap_remove(card_index);
                        player.three_up_cards.push(card);

                        // let all the players know about this card movement.
                        let _ = self.broadcast_messages_sender().send(
                            ServerMessage::MovePlayerCardFromHandToThreeUp {
                                hand_card_index: card_index,
                                player_id: client_id,
                            },
                        );

                        Ok(())
                    }
                    ClickedCardLocation::FromThreeUpCards { card_index } => {
                        // the player clicked a card from his 3 up cards during the 3 up selection, move
                        // that card to his hand.
                        let card_index = card_index as usize;
                        if card_index >= player.three_up_cards.len() {
                            return Err(GameServerError::NoSuchCard);
                        }
                        let card = player.three_up_cards.swap_remove(card_index);
                        player.cards_in_hand.push(card);

                        // let all the players know about this card movement.
                        let _ = self.broadcast_messages_sender().send(
                            ServerMessage::MovePlayerCardFromThreeUpToHand {
                                up_three_card_index: card_index,
                                player_id: client_id,
                            },
                        );

                        Ok(())
                    }
                    _ => {
                        // if the player clicked anything else during the 3 up selection, no need
                        // to do anything.
                        Ok(())
                    }
                }
            }
            LobbyState::GameStarted(in_game_state) => {
                let mut in_game_ctx = InGameLobbyContext::new(in_game_state, &mut self.data);
                in_game_ctx
                    .click_card(client_id, clicked_card_location)
                    .await
            }
        }
    }

    /// Stops the game, if the lobby is in game
    pub async fn stop_game(&mut self){
        match std::mem::replace(&mut self.state, LobbyState::Waiting){
            LobbyState::Waiting => {},
            LobbyState::Choosing3Up(chooosing_3_up_state) => {
                chooosing_3_up_state._timer.stop().await
            },
            LobbyState::GameStarted(InGameLobbyState{ current_turn, .. }) => {
                current_turn.stop_next_turn_timer().await;
            },
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
