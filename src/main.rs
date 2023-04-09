#![allow(dead_code)]
#![allow(unused_imports)]

mod piece;
mod board;
mod game;
mod ai;
use crate::ai::scrambled_board::ScrambledBoard;
// use crate::{piece::Piece, board::Board};

fn main() {
    game::play_game();
}
