use crate::{
    space::{Coord, Piece},
    ScrambledBoard,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::Hash,
};

#[derive(Debug, Clone, PartialEq)]
pub enum GridError {
    RowIndexOutOfBounds { idx_found: usize, board_size: usize },
    ColIndexOutOfBounds { idx_found: usize, board_size: usize },
    SpaceOccupied(Coord),
}

impl Display for GridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GridError::RowIndexOutOfBounds {
                idx_found,
                board_size,
            } => {
                write!(
                    f,
                    "Found row index {}, but board is of size {}",
                    idx_found, board_size
                )
            }
            GridError::ColIndexOutOfBounds {
                idx_found,
                board_size,
            } => {
                write!(
                    f,
                    "Found col index {}, but board is of size {}",
                    idx_found, board_size
                )
            }
            GridError::SpaceOccupied(Coord { row, col }) => {
                write!(f, "Space at row {}, col {} already occupied", row, col)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    pub size: usize,
    pub grid: Vec<Piece>,
}

impl Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.grid.hash(state);
    }
}

impl Board {
    pub fn new(size: usize) -> Self {
        let grid = vec![Piece::Empty; size * size];
        Self { size, grid }
    }

    fn flat_index(&self, Coord { row, col }: Coord) -> usize {
        row * self.size + col
    }

    pub fn piece_at(&self, c: Coord) -> Result<Piece, GridError> {
        if let Some(p) = self.grid.get(self.flat_index(c)) {
            Ok(*p)
        } else if c.row >= self.size {
            Err(GridError::RowIndexOutOfBounds {
                idx_found: c.row,
                board_size: self.size,
            })
        } else {
            Err(GridError::ColIndexOutOfBounds {
                idx_found: c.col,
                board_size: self.size,
            })
        }
    }

    pub fn place(&mut self, p: Piece, c: Coord) -> Result<(), GridError> {
        if self.piece_at(c)? != Piece::Empty {
            return Err(GridError::SpaceOccupied(c));
        }
        let idx = self.flat_index(c);
        self.grid[idx] = p;
        Ok(())
    }

    pub fn is_full(&self) -> bool {
        self.grid.iter().all(|&p| p != Piece::Empty)
    }

    pub fn invert(&mut self) {
        for p in self.grid.iter_mut() {
            *p = p.inverse();
        }
    }

    pub fn inverse(&self) -> Self {
        let mut inverted = self.clone();
        inverted.invert();
        inverted
    }

    pub fn has_win(&self, piece: Piece) -> bool {
        let remaining_rows: Vec<usize> = (0..self.size).collect();
        let remaining_cols: Vec<usize> = (0..self.size).collect();
        self.has_win_recursive(piece, &remaining_rows, &remaining_cols)
    }

    fn has_win_recursive(
        &self,
        possible_winner: Piece,
        remaining_rows: &[usize],
        remaining_cols: &[usize],
    ) -> bool {
        if remaining_rows.is_empty() {
            return true;
        }
        let row = remaining_rows[0];
        let remaining_rows = &remaining_rows[1..];

        for (i, &col) in remaining_cols.iter().enumerate() {
            if self.piece_at(Coord { row, col }).unwrap() == possible_winner {
                let mut remaining_cols = remaining_cols.to_owned();
                remaining_cols.remove(i);
                if self.has_win_recursive(possible_winner, remaining_rows, &remaining_cols) {
                    return true;
                }
            }
        }

        false
    }

    pub fn transpose(&mut self) {
        let size = self.size;
        for row_idx in 0..size {
            // for each index in row, starting from (but not including) the main diagonal
            for i in (row_idx * (size + 1) + 1)..(row_idx + 1) * size {
                let j = i / size + (i % size) * size;
                self.grid.swap(i, j)
            }
        }
    }

    fn transposed(&self) -> Self {
        let mut new_board = self.clone();
        new_board.transpose();
        new_board
    }

    pub fn standardize(&mut self) {
        let mut rows: Vec<&[Piece]> = self.grid.chunks(self.size).collect();
        rows.sort_unstable_by(Self::row_cmp);
        self.grid = rows.into_iter().flatten().copied().collect();

        self.transpose();

        let mut rows: Vec<&[Piece]> = self.grid.chunks(self.size).collect();
        rows.sort_unstable_by(Self::row_cmp);
        self.grid = rows.into_iter().flatten().copied().collect();

        let transposed = self.transposed();
        if transposed.grid < self.grid {
            self.grid = transposed.grid;
        }
    }

    pub fn fully_standardize(&mut self) {
        while !self.is_standard() {
            self.standardize();
        }
    }

    pub fn is_standard(&self) -> bool {
        ({
            let rows: Vec<&[Piece]> = self.grid.chunks(self.size).collect();
            rows.windows(2)
                .all(|w| Self::row_cmp(&w[0], &w[1]) != Ordering::Greater)
        }) && ({
            let transposed = self.transposed();
            (self.grid <= transposed.grid)
                && ({
                    let rows: Vec<&[Piece]> = transposed.grid.chunks(self.size).collect();
                    rows.windows(2)
                        .all(|w| Self::row_cmp(&w[0], &w[1]) != Ordering::Greater)
                })
        })
    }

    pub fn pretty(&self) -> String {
        let mut display_string = " ".to_string();

        for col_idx in 0..self.size {
            display_string += &format!("  {}", (b'A' + col_idx as u8) as char);
        }

        for (row_idx, row) in self.grid.chunks(self.size).enumerate() {
            display_string += &format!("\n{}", row_idx + 1);

            for this_piece in row {
                display_string += &format!("  {}", this_piece);
            }
        }

        display_string
    }

    pub fn row_cmp(left: &&[Piece], right: &&[Piece]) -> Ordering {
        if left == right {
            return Ordering::Equal;
        }

        let left_count = left.iter().filter(|&p| p == &Piece::O).count();
        let right_count = right.iter().filter(|&p| p == &Piece::O).count();
        match left_count.cmp(&right_count) {
            // if O's equal, move to next step of comparison
            Ordering::Equal => (),
            o => return o,
        };

        let left_count = left.iter().filter(|&p| p == &Piece::X).count();
        let right_count = right.iter().filter(|&p| p == &Piece::X).count();
        match left_count.cmp(&right_count) {
            // if X's equal and nonzero, use derived comparison
            Ordering::Equal => left.cmp(right),
            o => o,
        }
    }
}

impl<T: Into<Vec<Piece>>> From<T> for Board {
    fn from(grid: T) -> Self {
        let grid: Vec<Piece> = grid.into();
        let size = f32::from(grid.len() as u8).sqrt() as usize;
        assert_eq!(size * size, grid.len());

        Self { size, grid }
    }
}

impl From<ScrambledBoard> for Board {
    fn from(scrambled: ScrambledBoard) -> Self {
        let mut b = Self::new(scrambled.size);
        b.grid = scrambled.bare_grid();
        b
    }
}

impl From<&ScrambledBoard> for Board {
    fn from(scrambled: &ScrambledBoard) -> Self {
        let mut b = Self::new(scrambled.size);
        b.grid = scrambled.bare_grid();
        b
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.grid.chunks(self.size) {
            for this_piece in row {
                write!(f, "{}  ", this_piece)?;
            }
            write!(f, "\n  ")?;
        }

        write!(f, "  ")
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display_string = " ".to_string();

        for col_idx in 0..self.size {
            display_string += &format!("  {}", (b'A' + col_idx as u8) as char);
        }

        for (row_idx, row) in self.grid.chunks(self.size).enumerate() {
            display_string += &format!("\n{}", row_idx + 1);

            for this_piece in row {
                display_string += &format!("  {}", this_piece);
            }
        }

        write!(f, "{}  ", display_string)
    }
}
