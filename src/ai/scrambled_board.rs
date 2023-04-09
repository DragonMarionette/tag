use std::cmp::Ordering;
use std::fmt::Debug;

use crate::piece::Piece;
use crate::board::{Board, GridError};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Space {
    piece: Piece,
    row: usize,
    col: usize,
}

impl Debug for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}, {})", self.piece, self.row, self.col )
    }
}

#[derive(Clone, Debug)]
pub struct ScrambledBoard {
    size: usize,
    grid: Vec<Vec<Space>>,
}

impl ScrambledBoard {
    pub fn from_board(b: &Board) -> ScrambledBoard {
        let size = b.size;
        let mut grid = Vec::new();
        for row in 0..size {
            let mut this_row = Vec::new();
                for col in 0..size {
                    this_row.push(Space {piece: b.piece_at(row, col).unwrap(), row, col} );
                }
            grid.push(this_row);
        }

        ScrambledBoard { size, grid }
    }

    pub fn to_board(&self) -> Board {
        let mut b = Board::new(self.size);
        for space in self.spaces() {
            b.place(space.piece, space.row, space.col).unwrap();
        }
        b
    }

    pub fn to_board_scrambled(&self) -> Board {
        // let size = self.size;
        // let mut grid = Vec::new();

        // for row in 0..size {
        //     let mut this_row = Vec::new();
        //     for col in 0..size {
        //         this_row.push(self.piece_at(row, col).unwrap());
        //     }
        //     grid.push(this_row);
        // }
        // Board {size, grid}
        let mut b = Board::new(self.size);

        for row in 0..self.size {
            for col in 0..self.size {
                b.place(self.piece_at(row, col).unwrap(), row, col).unwrap();
            }
        }
        
        b
    }

    pub fn spaces(&self) -> std::iter::Flatten<std::slice::Iter<'_, Vec<Space>>>{
        self.grid.iter().flatten()
    }

    pub fn spaces_mut(&mut self) -> std::iter::Flatten<std::slice::IterMut<'_, Vec<Space>>>{
        self.grid.iter_mut().flatten()
    }

    pub fn piece_at(&self, row: usize, col: usize) -> Result<Piece, GridError> {
        if row >= self.size {
            return Err(GridError::RowIndexOutOfBounds {idx_found: row, board_size: self.size});
        }
        if col >= self.size {
            return Err(GridError::ColIndexOutOfBounds {idx_found: col, board_size: self.size});
        }
        Ok(self.grid[row][col].piece)
    }

    fn transpose(&mut self) {
        for row in 0..self.size {
            for col in 0..row {
                let row_col = self.grid[row][col];
                let col_row = self.grid[col][row];
                self.grid[row][col] = col_row;
                self.grid[col][row] = row_col;
            }
        }
    }

    fn transposed(&self) -> ScrambledBoard {
        let mut new_board = self.clone();
        new_board.transpose();
        new_board
    }
    
    // fn transpose_shallow(&mut self) {
    //     for space in self.spaces_mut(){
    //         let row = space.row;
    //         let col = space.col;
    //         space.col = row;
    //         space.row = col;
    //     }
    // }
    //
    // fn transposed_shallow(&self) -> ScrambledBoard {
    //     let mut new_board = self.clone();
    //     new_board.transpose_shallow();
    //     new_board
    // }

    pub fn invert(&mut self) {
        for space in self.spaces_mut() {
            space.piece = space.piece.inverse();
        }
    }

    pub fn inverse(&self) -> ScrambledBoard{
        let mut new_board = self.clone();
        new_board.invert();
        new_board
    }

    pub fn standardize(&mut self) {
        self.transpose();
        self.grid.sort_unstable_by(row_cmp);
        self.transpose();
        self.grid.sort_unstable_by(row_cmp);
    }

    pub fn standardized(&self) -> ScrambledBoard {
        let mut new_board = self.clone();
        new_board.standardize();
        new_board
    }

}


fn row_cmp(left: &Vec<Space>, right: &Vec<Space>) -> Ordering {
    let left_count = count(left, Piece::O);
    let right_count = count(right, Piece::O);
    match left_count.cmp(&right_count) {
        Ordering::Equal => if left_count == 0 {return Ordering::Equal;}, // if O's equal and nonzero, move to next step of comparison
        o => return o,
    };

    let left_count = count(left, Piece::X);
    let right_count = count(right, Piece::X);
    match left_count.cmp(&right_count) {
        Ordering::Equal => if left_count == 0 {return Ordering::Equal;}, // if X's equal and nonzero, move to next step of comparison
        o => return o,
    };

    match weight_positions(left, Piece::O).cmp(&weight_positions(right, Piece::O)) {
        Ordering::Equal => (), // if O's same, move to next step of comparison
        o => return o,
    };

    weight_positions(left, Piece::X).cmp(&weight_positions(right, Piece::X))
}

fn count(vec: &Vec<Space>, p: Piece) -> usize {
    vec.iter().filter(|&space| space.piece == p).count()
}

fn weight_positions(vec: &Vec<Space>, p: Piece) -> usize {
    vec.iter().enumerate().map(|(i, &space)| if space.piece == p {1 << i} else {0}).sum()
}