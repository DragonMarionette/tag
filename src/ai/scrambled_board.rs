use crate::piece::Piece;
use crate::board::{Board, GridError};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Space {
    piece: Piece,
    row: usize,
    col: usize,
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

    pub fn spaces(&self) -> std::iter::Flatten<std::slice::Iter<'_, Vec<Space>>>{
        self.grid.iter().flatten()
    }

    pub fn spaces_mut(&mut self) -> std::iter::Flatten<std::slice::IterMut<'_, Vec<Space>>>{
        self.grid.iter_mut().flatten()
    }

    pub fn piece_at(&mut self, row: usize, col: usize) -> Result<Piece, GridError> {
        if row >= self.size {
            return Err(GridError::RowIndexOutOfBounds {idx_found: row, board_size: self.size});
        }
        if col >= self.size {
            return Err(GridError::ColIndexOutOfBounds {idx_found: col, board_size: self.size});
        }
        Ok(self.grid[row][col].piece)
    }

    pub fn standardize(&mut self) { // TODO

    }

    pub fn transpose(&mut self) {
        // for row in 0..self.size {
        //         for col in 0..self.size {
        //             self.grid[row][col].row = col;
        //             self.grid[row][col].col = row;
        //         }
        // }
        for space in self.spaces_mut(){
            let row = space.row;
            let col = space.col;
            space.col = row;
            space.row = col;
        }
    }

    pub fn transposed(&self) -> ScrambledBoard {
        let mut new_board = self.clone();
        new_board.transpose();
        new_board
    }

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
}