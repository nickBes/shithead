use crate::{
    cards::{CardsDeck, Rank, CARDS_BY_ID},
    game_server::{ClientId, GameServerError},
    messages::{ClickedCardLocation, ServerMessage},
};

use super::{turn::Turn, LobbyNonStateData};

/// A state of an in game lobby.
#[derive(Debug)]
pub struct InGameLobbyState {
    pub deck: CardsDeck,
    pub trash: CardsDeck,
    pub current_turn: Turn,
}

impl InGameLobbyState {
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
}

/// An in game lobby context, which allows peforming operations on an in game lobby.
pub struct InGameLobbyContext<'a> {
    in_game_state: &'a mut InGameLobbyState,
    lobby_data: &'a mut LobbyNonStateData,
}

impl<'a> InGameLobbyContext<'a> {
    /// Creates a new in game lobby state wrapper.
    pub fn new(
        in_game_state: &'a mut InGameLobbyState,
        lobby_data: &'a mut LobbyNonStateData,
    ) -> Self {
        Self {
            in_game_state,
            lobby_data,
        }
    }

    /// Returns the current turn's player id.
    pub fn current_turn_player_id(&self) -> ClientId {
        self.in_game_state.current_turn.player_id()
    }

    /// Stops the current turn, advances the current turn to next player, and updates the players
    /// about the new turn.
    pub async fn advance_turn_and_update_players(&mut self) {
        let new_turn_player_id = self
            .lobby_data
            .player_list
            .find_next_turn_player_after(self.in_game_state.current_turn.player_id());

        // create a new turn for the next player and stop the previous turn's timer.
        let new_turn = Turn::new(self.lobby_data.id, new_turn_player_id);
        let previous_turn = std::mem::replace(&mut self.in_game_state.current_turn, new_turn);
        previous_turn.stop_next_turn_timer().await;

        // update all the players about this turn.
        let _ = self
            .lobby_data
            .broadcast_messages_sender
            .send(ServerMessage::Turn(new_turn_player_id));
    }

    /// The current turn has timed out, give the current turn player the trash or play one of his
    /// cards, and advance to the next turn.
    pub async fn turn_timeout(&mut self) {
        // give the player all the cards in the trash
        self.lobby_data.player_list.give_cards_to_player(
            self.in_game_state.current_turn.player_id(),
            self.in_game_state.trash.take_all(),
        );

        // let all the players in the lobby know that the player who timed out got all cards from
        // the trash.
        let _ = self
            .lobby_data
            .broadcast_messages_sender
            .send(ServerMessage::GiveTrash(
                self.in_game_state.current_turn.player_id(),
            ));

        self.advance_turn_and_update_players().await
    }

    /// The player has finished playing his turn, move on the next turn.
    pub async fn turn_finished(&mut self) {
        self.advance_turn_and_update_players().await;
    }

    /// Handles a card click from one of the clients.
    pub async fn click_card(
        &mut self,
        client_id: ClientId,
        clicked_card_location: ClickedCardLocation,
    ) -> Result<(), GameServerError> {
        let current_turn_player_id = self.in_game_state.current_turn.player_id();

        if current_turn_player_id != client_id {
            return Err(GameServerError::NotYourTurn);
        }

        let player = self
            .lobby_data
            .player_list
            .get_player(client_id)
            .ok_or(GameServerError::NotInALobby)?;

        match clicked_card_location {
            ClickedCardLocation::Trash => {
                // if the player can place any of his cards in the trash, then he is not allowed to
                // take the trash
                if player
                    .what_cards_can_be_placed_on(self.in_game_state.trash_top_card_rank())
                    .next()
                    .is_some()
                {
                    return Err(GameServerError::CantTakeTrashBecauseSomeCardsCanBePlayed);
                }

                // give the player all the cards in the trash
                self.lobby_data.player_list.give_cards_to_player(
                    current_turn_player_id,
                    self.in_game_state.trash.take_all(),
                );

                // let all the players in the lobby know that the player who timed out got all cards from
                // the trash.
                let _ = self
                    .lobby_data
                    .broadcast_messages_sender
                    .send(ServerMessage::GiveTrash(current_turn_player_id));


                // the player has finished his turn
                self.turn_finished().await;

                Ok(())
            }
            _ => todo!(),
        }
    }
}
