mod piece;
mod board;
mod game;
use crate::{piece::Piece, board::Board};

fn main() {
    game::play_game();
}
