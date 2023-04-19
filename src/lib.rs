pub mod game;
pub mod space;
pub mod players;

mod board;
pub use board::Board;

mod scrambled_board;
pub use scrambled_board::ScrambledBoard;

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        players::{AiLazy, AiParallel, AiSerial, Player},
        space::Piece,
    };

    #[test]
    fn analyze_ai_serial() {
        let size = 5;
        let mut ai_x = AiSerial::new(size, Piece::X, 100);
        let mut b = Board::new(size);
        ai_x.make_move(&mut b);
    }

    #[test]
    fn analyze_ai_parallel() {
        let size = 5;
        let mut ai_x = AiParallel::new(size, Piece::X);
        let mut b = Board::new(size);
        ai_x.make_move(&mut b);
    }

    #[test]
    fn analyze_ai_lazy() {
        let size = 5;
        let mut ai_x = AiLazy::new(size, Piece::X);
        let mut b = Board::new(size);
        ai_x.make_move(&mut b);
    }
}
