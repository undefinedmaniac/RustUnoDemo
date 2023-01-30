use std::fmt;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rand::distributions::{Distribution, Uniform};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color
{
    Red, Green, Blue, Yellow, Unpicked
}

impl fmt::Display for Color
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
    {
        write!(f, "{}", match *self {
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Blue => "Blue",
            Color::Yellow => "Yellow",
            Color::Unpicked => "Unpicked"
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CardType
{
    Number(u8), Skip, Reverse, DrawTwo,
    Wildcard, DrawFourWildcard
}

impl fmt::Display for CardType
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
    {
        let temp;
        write!(f, "{}", match *self {
            // CardType::Number(number) => match number {
            //     0 => "Zero",
            //     1 => "One",
            //     2 => "Two",
            //     3 => "Three",
            //     4 => "Four",
            //     5 => "Five",
            //     6 => "Six",
            //     7 => "Seven",
            //     8 => "Eight",
            //     9 => "Nine",
            //     _ => unreachable!()
            // },
            CardType::Number(number) => {
                temp = number.to_string();
                temp.as_str()
            },
            CardType::Skip => "Skip",
            CardType::Reverse => "Reverse",
            CardType::DrawTwo => "Draw 2",
            CardType::Wildcard => "Wildcard",
            CardType::DrawFourWildcard => "Draw 4 Wildcard"
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Card
{
    pub card_type: CardType,
    pub color: Color
}

impl Card
{
    pub fn new(card_type: CardType, color: Color) -> Card 
    {
        Card { card_type, color }
    }

    pub fn is_playable_on(&self, card: Card) -> bool
    {
        match (self.card_type, card.card_type) {
            // You can play a wildcard on any other card
            (CardType::Wildcard, _) | (CardType::DrawFourWildcard, _) |

            // If both cards are of any other type and have matching types
            (CardType::Skip, CardType::Skip) | 
            (CardType::Reverse, CardType::Reverse) | 
            (CardType::DrawTwo, CardType::DrawTwo) => true,

            // If both cards are of type Number and they have the same value
            (CardType::Number(value1), CardType::Number(value2)) if value1 == value2 => true,

            // If the colors of the cards match
            _ => self.color == card.color
        }
    }
}

impl fmt::Display for Card
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.color == Color::Unpicked {
            write!(f, "{}", self.card_type)
        } else {
            write!(f, "{} {}", self.color, self.card_type)
        }   
    }
}

struct InfiniteDeck
{
    rng: SmallRng,
    uniform: Uniform<u8>
}

impl InfiniteDeck
{
    fn new() -> InfiniteDeck 
    {
        Self {
            rng: SmallRng::from_entropy(),
            uniform: Uniform::new_inclusive(0, 107)
        }
    }

    fn draw(&mut self) -> Card 
    {
        let card_seed = self.uniform.sample(&mut self.rng);
        let card_type = match card_seed % 27 {
            0 => CardType::Number(0),
            seed @ 1..=9 => CardType::Number(seed),
            seed @ 10..=18 => CardType::Number(seed-9),
            19..=20 => CardType::Skip,
            21..=22 => CardType::Reverse,
            23..=24 => CardType::DrawTwo,
            25 => CardType::Wildcard,
            26 => CardType::DrawFourWildcard,
            _ => unreachable!()
        };

        let color;
        if let CardType::Wildcard | CardType::DrawFourWildcard = card_type {
            color = Color::Unpicked;
        } else {
            color = match card_seed / 27 {
                0 => Color::Red,
                1 => Color::Green,
                2 => Color::Blue,
                3 => Color::Yellow,
                _ => unreachable!()
            };
        }

        Card::new(card_type, color)
    }
}

pub struct Player
{
    name: String,
    cards: Vec<Card>
}

impl Player
{
    pub fn name(&self) -> &String 
    {
        &self.name
    }

    pub fn number_of_cards(&self) -> usize
    {
        self.cards.len()
    }
}

impl fmt::Display for Player
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}'s Cards:\n", self.name)?;
        for (index, card) in self.cards.iter().enumerate() {
            write!(f, "{}. {}\n", index + 1, card)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NotEnoughPlayers;

impl fmt::Display for NotEnoughPlayers
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        write!(f, "You cannot start the game until you have at least 2 players!")
    }
}

pub struct Lobby
{
    players: Vec<Player>
}

impl Lobby
{
    // Return false if the username is already taken
    pub fn add_player(&mut self, username: &str) -> bool
    {
        let username_available = !self.players.iter().any(|player| player.name == username);
        if username_available {
            self.players.push(Player { name: String::from(username), cards: Vec::with_capacity(7) });
        }
        username_available
    }

    pub fn number_of_players(&self) -> usize
    {
        self.players.len()
    } 

    // Return false if there are not at least two players
    pub fn start(self) -> Result<Game, NotEnoughPlayers>
    {
        if self.players.len() < 2 {
            Err(NotEnoughPlayers {})
        } else {
            let mut game = Game {
                players: self.players,
                current_player_idx: 0,
                turn_direction_reversed: false,
    
                deck: InfiniteDeck::new(),
                top_card: None,
            };

            game.start();
            Ok(game)
        }
    }
}

fn array_next_index(index: usize, length: usize, reversed: bool) -> usize {
    if reversed {
        if index == 0 { length - 1 } else { index - 1 }
    } else {
        if index == length - 1 { 0 } else { index + 1 }
    }
}

pub struct Game
{
    players: Vec<Player>,
    current_player_idx: usize,
    turn_direction_reversed: bool,

    deck: InfiniteDeck,
    top_card: Option<Card>
}

impl fmt::Display for Game
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let length = self.players.len();
        let mut index = self.current_player_idx;
        let reversed = self.turn_direction_reversed;

        if reversed {
            index = array_next_index(index, length, false);
        }

        for i in 1..=length {
            if reversed {
                match i {
                    1 => write!(f, "{} <- ", self.players[index].name())?,
                    i if i == length => write!(f, "[{}]", self.players[index].name())?,
                    _ => write!(f, "{} <- ", self.players[index].name())?
                }
            } else {
                match i {
                    1 => write!(f, "[{}] -> ", self.players[index].name())?,
                    i if i == length => write!(f, "{}", self.players[index].name())?,
                    _ => write!(f, "{} -> ", self.players[index].name())?
                }
            }
            index = array_next_index(index, length, false);
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum PlayError
{
    InvalidCardIndex,
    CardUnplayable
}

impl Game
{
    pub fn new() -> Lobby
    {
        Lobby { players: Vec::with_capacity(2) }
    }

    pub fn number_of_players(&self) -> usize
    {
        self.players.len()
    } 

    pub fn player(&self) -> &Player
    {
        &self.players[self.current_player_idx]
    }

    pub fn next_turn(&mut self)
    {
        self.current_player_idx = array_next_index(self.current_player_idx, 
            self.players.len(), self.turn_direction_reversed);
    }

    pub fn turn_direction(&self) -> &str
    {
        if self.turn_direction_reversed { "Counter Clockwise" } else { "Clockwise" }
    }

    pub fn reverse(&mut self) 
    {
        self.turn_direction_reversed = !self.turn_direction_reversed;
    }

    pub fn top_card(&self) -> Card
    {
        self.top_card.unwrap()
    }

    pub fn play(&mut self, card_index: usize) -> Result<(), PlayError>
    {
        let player = &mut self.players[self.current_player_idx];
        player.cards.get(card_index)
                    .ok_or(PlayError::InvalidCardIndex)
                    .and_then(|card| {
                        if card.is_playable_on(self.top_card.unwrap()) {
                            self.top_card = Some(*card);
                            Ok(())
                        } else {
                            Err(PlayError::CardUnplayable)
                        }
                    })?;

        player.cards.remove(card_index);
        Ok(())
    }

    pub fn draw_one(&mut self) -> Option<Card>
    {
        let card = self.deck.draw();
        if card.is_playable_on(self.top_card()) {
            // The card is playable so play it immediately
            self.top_card = Some(card);
            None
        } else {
            // The card is not playable so give it to the player
            self.players[self.current_player_idx].cards.push(card);
            Some(card)
        }
    }

    pub fn draw_multiple(&mut self, number_of_cards: u8)
    {
        let player = &mut self.players[self.current_player_idx];

        for _ in 0..number_of_cards {
            player.cards.push(self.deck.draw());
        }
    }

    pub fn set_wildcard_color(&mut self, color: Color) {
        if let Some(Card { card_type: x @ CardType::Wildcard | 
                                      x @ CardType::DrawFourWildcard, .. }) = self.top_card {
            self.top_card = Some(Card { card_type: x, color: color });
        }
    }

    fn start(&mut self)
    {
        // Deal 7 cards to each player
        for player in self.players.iter_mut() {
            for _ in 0..7 {
                player.cards.push(self.deck.draw());
            }
        }

        // Choose the starting player
        self.current_player_idx = rand::thread_rng().gen_range(0..self.players.len());

        // Grab a top card from the deck, but make sure it isn't a draw four wildcard
        while let None | Some(Card { card_type: CardType::DrawFourWildcard, .. }) = self.top_card {
            self.top_card = Some(self.deck.draw());
        }
    }
}
