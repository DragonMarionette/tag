pub mod board;
pub use board::Board;

pub mod game;

pub mod space;
// pub use space::Piece;
// pub use space::Coord;
// pub use space::Space;

pub mod players;

mod scrambled_board;
pub use scrambled_board::ScrambledBoard;
pub mod scrambled_board_flat;

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        players::{AiLazy, AiParallel, AiSerial, Player},
        space::Piece,
    };

    #[test]
    fn analyze_ai_serial() {
        let size = 4;
        let mut ai_x = AiSerial::new(size, Piece::X, 100);
        let mut b = Board::new(size);
        ai_x.make_move(&mut b);
    }

    #[test]
    fn analyze_ai_parallel() {
        let mut ai_x = AiParallel::new(Piece::X, 100);
        let mut b = Board::new(4);
        ai_x.make_move(&mut b);
    }

    #[test]
    fn analyze_ai_lazy() {
        let size = 4;
        let mut ai_x = AiLazy::new(size, Piece::X);
        let mut b = Board::new(size);
        ai_x.make_move(&mut b);
    }
}
