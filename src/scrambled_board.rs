use std::cmp::Ordering;

use crate::board::GridError;
use crate::Board;
use crate::space::{Coord, Space, Piece};


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScrambledBoard {
    pub size: usize,
    grid: Vec<Space>,
}

impl From<Board> for ScrambledBoard {
    fn from(b: Board) -> Self {
        let size = b.size;
        let mut grid = Vec::new();
        for (i, &piece) in b.flat().enumerate() {
            grid.push(Space {
                piece,
                row: i / size,
                col: i % size
            })
        }
        

        Self { size, grid }
    }
}

impl ScrambledBoard {
    pub fn bare_grid(&self) -> Vec<Piece> {
        self.grid.iter().map(|s| s.piece).collect()
    }

    pub fn to_original_board(&self) -> Board {
        let mut b = Board::new(self.size);
        for space in self.spaces() {
            b.place(space.piece, space.row, space.col).unwrap();
        }
        b
    }

    pub fn place(&mut self, p: Piece, c: Coord) -> Result<(), GridError> {
        if self.piece_at(c)? != Piece::Empty {
            return Err(GridError::SpaceOccupied { row: c.row, col: c.col });
        }
        self.grid[c.row*self.size + c.col].piece = p;
        Ok(())
    }

    pub fn spaces(&self) -> std::slice::Iter<'_, Space> {
        self.grid.iter()
    }

    pub fn spaces_mut(&mut self) -> std::slice::IterMut<'_, Space> {
        self.grid.iter_mut()
    }

    pub fn space_at(&self, coordinate: Coord) -> Result<Space, GridError> {
        let (row, col) = (coordinate.row, coordinate.col);
        if row >= self.size {
            return Err(GridError::RowIndexOutOfBounds {
                idx_found: row,
                board_size: self.size,
            });
        }
        if col >= self.size {
            return Err(GridError::ColIndexOutOfBounds {
                idx_found: col,
                board_size: self.size,
            });
        }
        Ok(self.grid[row*self.size + col])
    }

    pub fn piece_at(&self, coordinate: Coord) -> Result<Piece, GridError> {
        Ok(self.space_at(coordinate)?.piece)
    }

    pub fn transpose(&mut self) {
        let size = self.size;
        for row in 0..size {
            // for each index in row, starting from (but not including) the main diagonal
            for i in (row*(size+1) + 1)..(row+1)*size {
                let j = (i % size) * size + i / size;
                self.grid.swap(i, j)

            } 
        }
    }

    fn transposed(&self) -> Self {
        let mut new_board = self.clone();
        new_board.transpose();
        new_board
    }

    pub fn invert(&mut self) {
        for space in self.spaces_mut() {
            space.piece = space.piece.inverse();
        }
    }

    pub fn inverse(&self) -> Self {
        let mut new_board = self.clone();
        new_board.invert();
        new_board
    }

    pub fn standardize(&mut self) {
        let mut rows: Vec<&[Space]> = self.grid.chunks(self.size).collect();
        rows.sort_unstable_by(row_cmp);
        self.grid = rows.into_iter().flatten().map(|s| *s).collect();

        self.transpose();
        
        let mut rows: Vec<&[Space]> = self.grid.chunks(self.size).collect();
        rows.sort_unstable_by(row_cmp);
        self.grid = rows.into_iter().flatten().map(|s| *s).collect();

        let transposed = self.transposed();
        if self.bare_grid() > transposed.bare_grid() {
            *self = transposed;
        }
    }

    pub fn fully_standardize(&mut self) {
        while !self.is_standard() {
            self.standardize();
        }
    }

    pub fn standardized(&self) -> Self {
        let mut new_board = self.clone();
        new_board.standardize();
        new_board
    }

    pub fn is_standard(&self) -> bool {
        ({
            let rows: Vec<&[Space]> = self.grid.chunks(self.size).collect();
            rows.windows(2).all(|w| row_cmp(&w[0], &w[1]) != Ordering::Greater)
        })
        &&
        ({
            let transposed = self.transposed();
            (self.bare_grid() <= transposed.bare_grid()) && 
            ({
                let rows: Vec<&[Space]> = transposed.grid.chunks(self.size).collect();
                rows.windows(2).all(|w| row_cmp(&w[0], &w[1]) != Ordering::Greater)
            })
        })
    }

    pub fn into_standardized(mut self) -> Self {
        self.standardize();
        self
    }
}

// TODO: is_standard(b: &Board)

fn row_cmp(left: &&[Space], right: &&[Space]) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }

    let left_count = count(left, Piece::O);
    let right_count = count(right, Piece::O);
    match left_count.cmp(&right_count) {
        // if O's equal, move to next step of comparison
        Ordering::Equal => (),
        o => return o,
    };

    let left_count = count(left, Piece::X);
    let right_count = count(right, Piece::X);
    match left_count.cmp(&right_count) {
        // if X's equal and nonzero, use derived comparison
        Ordering::Equal => bare_row_cmp(left, right),
        o => o,
    }
}

fn count(row: &[Space], p: Piece) -> usize {
    row.iter().filter(|&space| space.piece == p).count()
}
 fn bare_row_cmp(left: &&[Space], right: &&[Space]) -> Ordering {
    let left: Vec<Piece> = left.iter().map(|&space| space.piece).collect();
    let right: Vec<Piece> = right.iter().map(|&space| space.piece).collect();
    left.cmp(&right)
 }