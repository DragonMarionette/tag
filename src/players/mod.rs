use crate::Board;
use crate::Piece;
use std::fmt::Display;


mod human;
pub use human::Human;

mod ai_serial;
pub use ai_serial::AiSerial;

mod ai_parallel;
pub use ai_parallel::AiParallel;


pub trait Player: Display {
    fn make_move(&mut self, game_board: &mut Board);
    fn piece(&self) -> Piece;
}
