use std::fmt::Display;

use super::Player;
use crate::user_input;
use crate::{
    Board,
    space::Piece,
};

pub struct Human {
    pub name: String,
    pub piece: Piece,
}

impl Human {
    pub fn new(name: &str, piece: Piece) -> Self {
        Self {
            name: name.to_string(),
            piece,
        }
    }
}

impl Display for Human {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.piece.colorize(&self.name))
    }
}

impl Player for Human {
    fn piece(&self) -> Piece {
        self.piece
    }
    fn make_move(&mut self, game_board: &mut Board) {
        let c = user_input::get_move(&self.name, game_board);
        game_board.place(self.piece, c).expect("Move was not validated properly");
    }
}
