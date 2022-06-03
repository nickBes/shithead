use crate::lobby::lobby_player::LobbyPlayer;
use crate::lobby::CardId;
use crate::lobby::ClientId;
use std::collections::HashMap;
use std::ops::Index;

/// A list of players in the lobby, ordered by their turns.
#[derive(Debug)]
pub struct OrderedLobbyPlayers {
    players_by_id: HashMap<ClientId, LobbyPlayer>,
    turns_order: Vec<ClientId>,
}

impl OrderedLobbyPlayers {
    pub fn new() -> Self {
        Self {
            players_by_id: HashMap::new(),
            turns_order: Vec::new(),
        }
    }

    pub fn insert(&mut self, player_id: ClientId, lobby_player: LobbyPlayer) {
        self.players_by_id.insert(player_id, lobby_player);
        self.turns_order.push(player_id);
    }

    pub fn remove(&mut self, player_id: ClientId) -> Option<LobbyPlayer> {
        let result = self.players_by_id.remove(&player_id);
        if result.is_some() {
            // if there was a player with this id, remove this player from the turns order vector as well
            self.turns_order
                .retain(|&player_id_to_check| player_id_to_check != player_id);
        }
        result
    }

    pub fn len(&self) -> usize {
        self.turns_order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.turns_order.is_empty()
    }

    pub fn player_ids<'a>(&'a self) -> impl Iterator<Item = ClientId> + 'a {
        self.turns_order.iter().copied()
    }

    pub fn players<'a>(
        &'a mut self,
    ) -> std::collections::hash_map::ValuesMut<'a, ClientId, LobbyPlayer> {
        self.players_by_id.values_mut()
    }

    /// Finds the id of the player that will play after the player with the given id, according to
    /// the turns order.
    pub fn find_next_turn_player_after(&self, player_id: ClientId) -> ClientId {
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
    pub fn first_turn_player(&self) -> ClientId {
        self.turns_order[0]
    }

    /// Gives cards to a player.
    pub fn give_cards_to_player(
        &mut self,
        player_id: ClientId,
        cards: impl Iterator<Item = CardId>,
    ) {
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
