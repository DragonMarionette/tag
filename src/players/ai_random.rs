use std::fmt::Display;
use crate::Board;
use crate::space::Piece;
use super::{Player, available_spaces};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::rngs::ThreadRng;

pub struct AiRandom {
    piece: Piece,
    rng: ThreadRng,
}

impl Display for AiRandom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.piece.colorize("AI"), self.piece)
    }
}

impl Player for AiRandom {
    fn make_move(&mut self, game_board: &mut Board) {
        let spaces = available_spaces(&game_board);
        let chosen_move = spaces.choose(&mut self.rng).unwrap();
        game_board
            .place(self.piece(), chosen_move.row, chosen_move.col)
            .unwrap();
    }

    fn piece(&self) -> Piece {
        self.piece
    }
}

impl AiRandom {
    pub fn new(piece: Piece) -> Self {
        Self {piece, rng: thread_rng()}
    }
}