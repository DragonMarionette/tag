use std::io;
use std::io::Write;
use regex::Regex;
use crate::{piece::Piece, board::{Board, PlacementError}};

pub fn play_game() {    
    let mut game_board = Board::new(get_board_size());
    println!("\n{}\n\n", game_board.pretty());

    let mut current_player = Piece::X;

    for _ in 0..(game_board.size * game_board.size) {
        make_move(current_player, &mut game_board);
        
        println!("\n{}\n\n", game_board.pretty());

        if game_board.has_win(current_player) {
            println!("{} wins!", current_player);
            return;
        }
        current_player = current_player.inverse();
    }

    println!("It's a tie!")
}

fn get_board_size() -> usize { // TODO: Write to be cleaner using match or something?
    let mut input = String::new();
    print!("Enter a size for the board: ");
    std::io::stdout().flush().unwrap();  // guarantee that the above print is written to console

    if io::stdin().read_line(&mut input).is_err() {
        println!("Unable to read your input. Try again.");
        return get_board_size();
    }

    let size_str = input.trim();
    let size_parsed = size_str.parse::<usize>();
    if size_parsed.is_err() {
        println!("\"{}\" is not a valid board size. Please enter a positive integer.", size_str);
        return get_board_size();
    }

    let size = size_parsed.unwrap();
    if size > 9 {
        println!("Board too large. Choose a size up to 9.");
        return get_board_size();
    }

    size
}

fn make_move(player: Piece, game_board: &mut Board) {
    let mut input = String::new();
    print!("{}, enter your move: ", player);
    std::io::stdout().flush().unwrap();  // guarantee that the above print is written to console

    if io::stdin().read_line(&mut input).is_err() {
        println!("Unable to read your input. Try again.");
        return make_move(player, game_board);
    }

    input = input.to_uppercase();
    let move_str = input.trim();
    let move_validator = Regex::new(r"^[A-Z]\d$").unwrap(); // TODO: Compile only once using lazystatic, or check w/out regex
    if !move_validator.is_match(move_str) {
        println!("Invalid space. Enter in the form \"A1\"");
        return make_move(player, game_board);
    }

    let col = move_str.chars().nth(0).unwrap() as u8 - 'A' as u8; // TODO: These feel ugly. More idiomatic way to do it?
    let row = move_str.chars().nth(1).unwrap().to_digit(10).unwrap() - 1;
    
    match game_board.place(player, row as usize, col as usize) {
        Err(PlacementError::RowIndexOutOfBounds {..}) => {
            println!("Invalid space, row out of bounds. Try again.");
            make_move(player, game_board)
        },
        Err(PlacementError::ColIndexOutOfBounds {..}) => {
            println!("Invalid space, column out of bounds. Try again.");
            make_move(player, game_board)
        },
        Err(PlacementError::SpaceOccupied {..}) => {
            println!("Space {} is already filled. Try again.", move_str);
            make_move(player, game_board)
        },
        Ok(_) => ()
    }
}