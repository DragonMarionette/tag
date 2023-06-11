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
    play();
    // test_iterator();
    // play_vs_ground_up();
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

    let count_x = board_size * board_size / 3;
    let count_o = count_x;
    let count_empty = board_size * board_size - count_x - count_o;

    let all_boards = tag::players::ai_ground_up::BoardIterator::from_counts(
        board_size,
        count_x,
        count_o,
        count_empty,
    );
    let filtered_boards = all_boards
        .clone()
        .filter(|b| tag::ScrambledBoard::from(b.clone()).is_standard());

    println!("{} boards of size {}", all_boards.count(), board_size);
    println!(
        "{} standard boards of size {}",
        filtered_boards.clone().count(),
        board_size
    );

    for (i, b) in filtered_boards.enumerate() {
        println!("\n{}:", i);
        println!("{}", b);
    }
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
