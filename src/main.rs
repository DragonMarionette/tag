#![allow(dead_code)]
#![allow(unused_imports)]

use ai::ai::AI;

mod piece;
mod board;
mod game;
mod ai;
// use crate::{piece::Piece, board::Board};

fn main() {
    let ai_opponent = AI::new(piece::Piece::X, 100);
    game::play_human_vs(ai_opponent);
}
