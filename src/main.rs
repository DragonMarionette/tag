#![allow(dead_code)]
// #![allow(unused_imports)]

use ai::ai::AI;

mod piece;
mod board;
mod game;
mod ai;
// use crate::{piece::Piece, board::Board};

fn main() {
    // let ai_opponent = AI::new(piece::Piece::X, 100);
    // game::play_human_vs(ai_opponent);

    let ai_x = AI::new(piece::Piece::X, 100);
    let ai_o = AI::new(piece::Piece::O, 100);
    game::play_ai_vs_ai(ai_x, ai_o);
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::piece::Piece;
    use crate::ai::ai::AI;

    #[test]
    fn analyze_test() {
        let mut ai_x = AI::new(Piece::X, 100);
        let mut b = Board::new(4);
        ai_x.make_move(Piece::X, &mut b);
    }
}
