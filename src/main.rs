use game::{Color, Game};
use std::io::{self, Write};
use crate::game::{CardType, PlayError};

mod game;

fn get_next_line() -> String
{
    let mut line = String::new();
    io::stdin().read_line(&mut line).expect("Can't read from standard input :(");
    line
}

fn print_and_flush(text: &str)
{
    print!("{text}");
    std::io::stdout().flush().expect("Error while flushing stdout");
}

fn skip_turn(game: &mut Game) 
{
    println!("{} had their turn skipped!\n", game.player().name());
    game.next_turn();
}

fn reverse(game: &mut Game) 
{
    game.reverse();
    println!("Reversing the turn direction! The new direction is {}\n\
    New turn order: {}\n", game.turn_direction(), game);
}

fn draw(game: &mut Game, number_of_cards: u8) 
{
    debug_assert_ne!(number_of_cards, 1);
    println!("{} drew {} cards", game.player().name(), number_of_cards);
    game.draw_multiple(number_of_cards);
}

fn pick_wildcard_color(game: &mut Game)
{
    let color: Color = loop {
        print_and_flush("Select a color for the wildcard:\n\
        1 - Red\n\
        2 - Green\n\
        3 - Blue\n\
        4 - Yellow\n\
        Your choice: ");
    
        let choice = get_next_line();
        break match choice.trim() {
            "1" => Color::Red,
            "2" => Color::Green,
            "3" => Color::Blue,
            "4" => Color::Yellow,
            _ => {
                println!("Enter a value between 1 and 4!\n");
                continue
            }
        };
    };

    println!("The wildcard color is now {}\n", color);
    game.set_wildcard_color(color);
}

fn main() 
{
    let mut game = Game::new();

    println!("To start the game, you must add at least 2 players, then select 'start'\n");

    loop {
        if game.number_of_players() >= 2 {
            print_and_flush("Select an option:\n\
            1. Add a player\n\
            2. Start the game\n\
            Choose an option: ");

            let choice = get_next_line();
            match choice.trim() {
                "1" => println!(),
                "2" => break,
                _ =>  {
                    println!("Please enter an option in the range 1 - 2!\n");
                    continue
                }
            }
        }

        loop {
            print_and_flush("Enter a username: ");
            let username = get_next_line().trim().to_owned();
            if game.add_player(&username) {
                println!("Added player {}!\n", username);
                break;
            }

            println!("Username '{}' is already taken. Please choose a different username\n", username);
        }
    }

    let mut game = game.start().unwrap();
    println!("\nStarting the game! The starting player is {}\n\
    Turn order: {}\n\n\
    The top card is a {}\n", game.player().name(), game, game.top_card());

    if match game.top_card().card_type {
        CardType::Skip => { 
            skip_turn(&mut game); 
            true
        },
        CardType::Reverse => { 
            reverse(&mut game); 
            skip_turn(&mut game); 
            true
        },
        CardType::DrawTwo => { 
            draw(&mut game, 2); 
            skip_turn(&mut game); 
            true
        },
        CardType::Wildcard => {
            println!("{}", game.player());
            pick_wildcard_color(&mut game);
            false
        },
        _ => false
    }
    {
        println!("The new starting player is {}\n", game.player().name());
    }

    loop {
        let player = game.player();

        print_and_flush(format!("\
        It's {}'s turn!\n\
        The top card is a {}\n\n\
        {}\
        Choose a card or type 'draw': ", 
        player.name(), game.top_card(), player).as_str());

        let result = match get_next_line().trim().to_lowercase().as_str() {
            "draw" => {
                match game.draw_one() {
                    Some(card) => { println!("You drew a {}! It's not playable on the current card!", card); Ok(false) }
                    None => { println!("You drew a {}! It's playable on the current card!", game.top_card()); Ok(true) }
                }
            }
            text => {
                text.parse::<usize>()
                    .map_err(|_| PlayError::InvalidCardIndex)
                    .and_then(|choice| game.play(choice - 1))
                    .map(|_| true)
            }
        };

        let player = game.player();
        if result.is_err() {
            match result.unwrap_err() {
                PlayError::InvalidCardIndex =>
                    println!("Please enter a card index in the range 1 - {}, or type 'draw' to draw\n", player.number_of_cards()),
                PlayError::CardUnplayable =>
                    println!("The card you picked cannot be played on a {}. \
                    Select a different card or choose the 'draw' option\n", game.top_card())
            }
            continue;
        }

        if result.unwrap() {
            println!("{} played a {}!\n", player.name(), game.top_card());

            if player.number_of_cards() == 0 {
                println!("{} has played their last card! They are the winner!\n", player.name());
                break;
            }

            match game.top_card().card_type {
                CardType::Reverse => reverse(&mut game),
                CardType::Wildcard | CardType::DrawFourWildcard => pick_wildcard_color(&mut game),
                _ => ()
            }

            game.next_turn();

            match game.top_card().card_type {
                CardType::Skip => skip_turn(&mut game),
                CardType::Reverse if game.number_of_players() == 2 => skip_turn(&mut game),
                CardType::DrawTwo => { 
                    draw(&mut game, 2); 
                    skip_turn(&mut game); 
                },
                CardType::DrawFourWildcard => {
                    draw(&mut game, 4); 
                    skip_turn(&mut game); 
                }
                _ => ()
            }
        } else {
            println!("{} was unable to play a card! Their turn is over\n", player.name());
            game.next_turn();
        }
    }

    print_and_flush("Press enter to close the program...");
    get_next_line();
}
