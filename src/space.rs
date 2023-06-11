use std::fmt::{Debug, Display};
use colored::{Colorize, ColoredString};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Piece {
    X,
    O,
    Empty
}

impl Piece {
    pub fn inverse(&self) -> Self {
        match self {
            Self::X => Self::O,
            Self::O => Self::X,
            Self::Empty => Self::Empty
        }
    }
    
    pub fn colorize(&self, s: &str) -> ColoredString {
        match self {
            Self::X => s.blue(),
            Self::O => s.green(),
            Self::Empty => s.normal()
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece_as_str = match self {
            Self::X => Self::X.colorize("X"),
            Self::O => Self::O.colorize("O"),
            Self::Empty => Self::Empty.colorize("·")
        };
        write!(f, "{}", piece_as_str)
    }
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Coord {
    pub row: usize,
    pub col: usize,
}


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Space {
    pub piece: Piece,
    pub coord: Coord
}

impl Debug for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}, {})", self.piece, self.coord.row, self.coord.col)
    }
}