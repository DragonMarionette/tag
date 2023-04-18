use std::io;
use std::io::Write;
use crate::{space::Piece, Board, players::Player};


pub fn play_game(p1: &mut impl Player, p2: &mut impl Player, board_size: usize) {
    assert_ne!(p1.piece(), Piece::Empty);
    assert_ne!(p2.piece(), Piece::Empty);
    assert_ne!(p1.piece(), p2.piece());

    let mut game_board = Board::new(board_size);
    println!("\n{}\n\n", game_board.pretty());

    // this is neccessary because p1 and p2 may be of different types but can both be stored in current_player
    let mut p1: Box<&mut dyn Player> = Box::new(p1);
    let mut p2: Box<&mut dyn Player> = Box::new(p2);

    for turn in (1..=2).cycle() {
        if game_board.is_full() {
            println!("It's a tie!");
            return;
        }

        let current_player = match turn {
            1 => &mut p1,
            2 => &mut p2,
            _ => {panic!("turn was neither 1 nor 2, which should never happen");}
        };

        current_player.make_move(&mut game_board);
        
        println!("\n{}\n\n", game_board.pretty());

        if game_board.has_win(current_player.piece()) {
            println!("{} wins!", current_player);
            return;
        }
    }
}


pub fn get_board_size() -> usize {
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
