#![allow(unused_imports)]
use std::collections::VecDeque;
use tag::ScrambledBoard;
use tag::{
    game,
    players::{AiGroundUp, AiLazy, AiParallel, AiSerial, Human, Player},
    space::Piece,
    user_input, Board,
};

fn main() {
    // play();
    // test_iterator();
    play_vs_ground_up();
//     let b = Board::from(VecDeque::from(
//         vec![
//             Piece::Empty, Piece::X, Piece::X,
//             Piece::O, Piece::Empty, Piece::Empty,
//             Piece::X, Piece::O, Piece::O
//         ]
//     ));
//     println!("b =\n{}", b);
//     let sb = ScrambledBoard::from(b);
//     println!("sb.is_standard = {}", sb.is_standard());
//     println!("if standardized, b woud look like:\n{}", Board::from(sb.standardized()));
}

#[allow(dead_code)]
fn play() {
    let board_size = user_input::get_board_size();
    println!();

    let mut p1 = user_input::get_player(Piece::X, board_size);
    println!();

    let mut p2 = user_input::get_player(Piece::O, board_size);
    println!();

    match game::play_game(&mut p1, &mut p2, board_size) {
        game::GameState::P1 => {
            println!("{} wins!", p1)
        }
        game::GameState::P2 => {
            println!("{} wins!", p2)
        }
        game::GameState::Tie => {
            println!("It's a tie!")
        }
    }
}

#[allow(dead_code)]
fn test_iterator() {
    let board_size = user_input::get_board_size();
    println!();

    let all_boards = tag::players::ai_ground_up::BoardIterator::new(board_size, Piece::X);
    let filtered_boards = all_boards
        .clone()
        .filter(|b| tag::ScrambledBoard::from(b.clone()).is_standard());

    println!(
        "{} boards of size {}",
        all_boards.clone().count(),
        board_size
    );
    println!(
        "{} standard boards of size {}",
        filtered_boards.clone().count(),
        board_size
    );

    // for (i, b) in filtered_boards.enumerate() {
    //     println!("\n{}:", i);
    //     println!("{}", b);
    // }
}

#[allow(dead_code)]
fn play_vs_ground_up() {
    let board_size = user_input::get_board_size();
    println!();

    let mut p1: Box<dyn Player> = Box::new(AiGroundUp::new(board_size, Piece::X));

    let mut p2: Box<dyn Player> = Box::new(Human::new("Dan", Piece::O));
    println!();

    match game::play_game(&mut p1, &mut p2, board_size) {
        game::GameState::P1 => {
            println!("{} wins!", p1)
        }
        game::GameState::P2 => {
            println!("{} wins!", p2)
        }
        game::GameState::Tie => {
            println!("It's a tie!")
        }
    }
}
