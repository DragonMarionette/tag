use std::fmt::Display;
use colored::Colorize;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Piece {
    X,
    O,
    Empty
}

impl Piece {
    pub fn inverse(&self) -> Piece {
        match self {
            Piece::X => Piece::O,
            Piece::O => Piece::X,
            Piece::Empty => Piece::Empty
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece_as_str = match self {
            Piece::X => "X".blue(),
            Piece::O => "O".green(),
            Piece::Empty => "·".normal()
        };
        write!(f, "{}", piece_as_str)
    }
}
