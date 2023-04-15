use std::fmt::Display;
use colored::{Colorize, ColoredString};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
