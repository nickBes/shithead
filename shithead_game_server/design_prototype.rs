use std::collections::HashMap;

enum ClientRequest{
    ClickCard(ClickedCard),
}

enum ClickedCard{
    Deck,
    OfPlayer{
        player_id: (),
        card_index: u32,
    }
}

struct CardId(pub usize);
struct PlayerId(pub usize);

struct Game{
    deck: HashMap<CardId, Card>,
    players: HashMap<PlayerId, GamePlayer>,
    trash: HashMap<CardId, Card>,
    card_locations: HashMap<CardId, CardLocation>,
}

enum CardLocation{
    Deck,
    Trash,
    PlayersHand,
    PlayersThreeDown,
    PlayersThreeUp,
}

struct GamePlayer{
    cards_in_hand: HashMap<CardId, Card>,
    three_down_cards: HashMap<CardId, Card>,
    three_up_cards: HashMap<CardId, Card>,
}

struct Card{
    number: (),
    suit: (),
}




