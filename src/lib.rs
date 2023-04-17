pub mod board;
pub use board::Board;

pub mod game;

mod piece;
pub use piece::Piece;

pub mod players;

pub mod scrambled_board;

#[cfg(test)]
mod tests {
    use crate::{
        players::{Player, AiSerial, AiParallel},
        Piece,
        board::Board,
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
}
