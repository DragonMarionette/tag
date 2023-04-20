use std::io;
use std::io::Write;
use crate::{space::Piece, Board, players::{Player, Human, AiParallel, AiSerial}};

#[derive(Clone, Copy)]
pub enum Winner {
    P1,
    P2,
    Tie
}

pub fn play_game(p1: &mut Box<dyn Player>, p2: &mut Box<dyn Player>, board_size: usize) -> Winner {
    assert_ne!(p1.piece(), Piece::Empty);
    assert_ne!(p2.piece(), Piece::Empty);
    assert_ne!(p1.piece(), p2.piece());

    let mut game_board = Board::new(board_size);
    println!("\n{}\n\n", game_board.pretty());

    for turn in [Winner::P1, Winner::P2].iter().cycle() {
        if game_board.is_full() {
            return Winner::Tie;
        }

        let current_player = match turn {
            Winner::P1 => &mut *p1,
            Winner::P2 => &mut *p2,
            _ => {panic!("turn was neither P1 nor P2, which should never happen");}
        };

        current_player.make_move(&mut game_board);
        
        println!("\n{}\n\n", game_board.pretty());

        if game_board.has_win(current_player.piece()) {
            return *turn;
        }
    }
    panic!("Loop ended, which should never happen")
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

pub fn get_player(piece: Piece, size: usize) -> Box<dyn Player> {
    println!("Choose a player to be {}. Options:", piece);
    println!("0\tHuman");
    println!("1\tLimited-lookahead AI");
    println!("2\tUnpredictable perfect AI");
    println!("3\tMore Predictable (but still perfect) AI");
    println!("4\tDeterministic Perfect AI");
    let mut input = String::new();
    print!("Your choice: ");
    std::io::stdout().flush().unwrap();  // guarantee that the above print is written to console

    let mut input_or_err = io::stdin().read_line(&mut input);


    while input_or_err.is_err()
    || input.trim().parse::<usize>().is_err()
    || input.trim().parse::<usize>().unwrap() > 4 {
        println!("You must enter a number from 0 to 4.");
        print!("Your choice: ");
        std::io::stdout().flush().unwrap();  // guarantee that the above print is written to console
        input.clear();
        input_or_err = io::stdin().read_line(&mut input);
    }

    match input.trim().parse::<usize>().unwrap() {
        0 => {
            Box::new(Human::new(&get_name(piece), piece))
        },
        1 => {
            let mut new_player = Box::new(AiSerial::new(size, piece, get_depth(size)));
            new_player.load_strategy();
            new_player
        },
        2 => {
            let mut new_player = Box::new(AiSerial::new(size, piece, usize::MAX));
            new_player.load_strategy();
            new_player
        },
        3 => {
            let mut new_player = Box::new(AiParallel::new(size, piece, false));
            new_player.load_strategy();
            new_player
        },
        4 => {
            let mut new_player = Box::new(AiParallel::new(size, piece, true));
            new_player.load_strategy();
            new_player
        },
        _ => panic!("Recieved an illegal input that should already have been handled")
    }
}

fn get_depth(size: usize) -> usize {
    let mut input = String::new();
    print!("Enter a depth for AI analysis: ");
    std::io::stdout().flush().unwrap();  // guarantee that the above print is written to console

    if io::stdin().read_line(&mut input).is_err() {
        println!("Unable to read your input. Try again.");
        return get_board_size();
    }

    let depth_str = input.trim();
    let depth_parsed = depth_str.parse::<usize>();
    if depth_parsed.is_err() {
        println!("\"{}\" is not a valid board size. Please enter a positive integer.", depth_str);
        return get_board_size();
    }

    let depth = depth_parsed.unwrap();
    if depth > size*size {
        println!("Please enter a smaller number. There are only {} moves on a board of size {}.", size*size, size);
        return get_board_size();
    }

    depth
}

fn get_name(piece: Piece) -> String {
    let mut input = String::new();
    print!("Enter a name for player {}: ", piece);
    std::io::stdout().flush().unwrap();  // guarantee that the above print is written to console

    if io::stdin().read_line(&mut input).is_err() {
        println!("Unable to read your input. Try again.");
        return get_name(piece);
    }

    input.trim().to_string()
}
