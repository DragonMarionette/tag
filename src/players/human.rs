use std::io::Write;
use std::{fmt::Display, io};

use super::Player;
use crate::{
    board::{Board, GridError},
    piece::Piece,
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
        let mut input = String::new();
        print!("{}, enter your move: ", self);
        io::stdout().flush().unwrap(); // guarantee that the above print is written to console

        if io::stdin().read_line(&mut input).is_err() {
            println!("Unable to read your input. Try again.");
            return self.make_move(game_board);
        }

        input = input.to_uppercase();
        let move_str = input.trim();
        let move_validator = regex::Regex::new(r"^[A-Z]\d$").unwrap(); // TODO: Compile only once using lazystatic, or check w/out regex
        if !move_validator.is_match(move_str) {
            println!("Invalid space. Enter in the form \"A1\"");
            return self.make_move(game_board);
        }

        let mut characters = move_str.chars();
        let col = characters.next().unwrap() as u8 - b'A'; // TODO: These feel ugly. More idiomatic way to do it?
        let row = characters.next().unwrap().to_digit(10).unwrap() - 1;

        match game_board.place(self.piece, row as usize, col as usize) {
            Err(GridError::RowIndexOutOfBounds { .. }) => {
                println!("Invalid space, row out of bounds. Try again.");
                self.make_move(game_board)
            }
            Err(GridError::ColIndexOutOfBounds { .. }) => {
                println!("Invalid space, column out of bounds. Try again.");
                self.make_move(game_board)
            }
            Err(GridError::SpaceOccupied { .. }) => {
                println!("Space {} is already filled. Try again.", move_str);
                self.make_move(game_board)
            }
            Ok(_) => (),
        }
    }
}
