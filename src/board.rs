use std::fmt::Display;
use crate::piece::Piece;


#[derive(Debug, Clone)]
pub enum PlacementError {
    RowIndexOutOfBounds {idx_found: usize, board_size: usize},
    ColIndexOutOfBounds {idx_found: usize, board_size: usize},
    SpaceOccupied {row: usize, col: usize}
}

impl Display for PlacementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlacementError::RowIndexOutOfBounds {idx_found, board_size} => {
                write!(f, "Found row index {}, but board is of size {}", idx_found, board_size)
            },
            PlacementError::ColIndexOutOfBounds {idx_found, board_size} => {
                write!(f, "Found col index {}, but board is of size {}", idx_found, board_size)
            },
            PlacementError::SpaceOccupied { row: row_idx, col: col_idx } => {
                write!(f, "Space at row {}, col {} already occupied", row_idx, col_idx)
            },
        }
    }
}


#[derive(Debug, Clone)]
pub struct Board {
    pub size: usize,
    grid: Vec<Vec<Piece>>
}

impl Board {
    pub fn new(size: usize) -> Board {
        let grid = vec![ vec![Piece::Empty ; size] ; size ];
        Board {size, grid}
    }
    
    pub fn piece_at(&self, row:usize, col:usize) -> Result<Piece, PlacementError> {
        if row >= self.size {
            return Err(PlacementError::RowIndexOutOfBounds {idx_found: row, board_size: self.size});
        }
        if col >= self.size {
            return Err(PlacementError::ColIndexOutOfBounds {idx_found: col, board_size: self.size});
        }
        Ok(self.grid[row][col])
    }
    
    pub fn place(&mut self, p: Piece, row:usize, col:usize) -> Result<(), PlacementError> {
        if self.piece_at(row, col)? != Piece::Empty {
            return Err(PlacementError::SpaceOccupied { row, col });
        }
        self.grid[row][col] = p;
        Ok(())
    }

    pub fn invert(&mut self) -> () {
        for row in 0..self.size {
            for col in 0..self.size {
                self.grid[row][col] = self.grid[row][col].inverse();
            }
        }
    }

    pub fn has_win(&self, piece: Piece) -> bool {
        let remaining_rows: Vec<usize> = (0..self.size).collect();
        let remaining_cols = remaining_rows.clone();
        self.has_win_recurrent(piece, &remaining_rows, &remaining_cols)
    }

    fn has_win_recurrent(&self, piece: Piece, remaining_rows: &Vec<usize>, remaining_cols: &Vec<usize>) -> bool {
        if remaining_rows.is_empty() {
            return true;
        }
        let row = remaining_rows[0];
        let remaining_rows = remaining_rows[1..].to_vec();

        for (i, &col) in remaining_cols.iter().enumerate() {
            if self.piece_at(row, col).unwrap() == piece {
                let mut remaining_cols = remaining_cols.clone();
                remaining_cols.remove(i);
                if self.has_win_recurrent(piece, &remaining_rows, &remaining_cols) {
                    return true;
                }
            }
        }

        return false;
    }

    pub fn pretty(&self) -> String {
        let mut display_string = " ".to_string();

        for col in 0..self.size {
            display_string += &format!("  {}", ('A' as u8 + col as u8) as char);
        }

        for row in 0..self.size {
            display_string += &format!("\n{}", row + 1);

            for col in 0..self.size {
                let this_piece = &self.grid[row][col];
                display_string += &format!("  {}", this_piece);
            }
        }

        display_string
    }
}


// Does not include row/column indices
impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display_string = "".to_string();

        for row in 0..self.size {
            for col in 0..self.size {
                let this_piece = &self.grid[row][col];
                display_string += &format!("{}  ", this_piece);
            }
            display_string += "\n";
        }

        write!(f, "{}  ", display_string)
    }
}