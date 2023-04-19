#[allow(unused_imports)]
use tag::{
    game,
    players::{AiLazy, AiParallel, AiSerial, Human, Player},
    Board, space::Piece,
};

fn main() {
    play();
    // let size = 4;
    // let mut ai_x = AiSerial::new(size, Piece::X, 100);
    // let mut ai_x = AiLazy::new(size, Piece::X);
    // let mut ai_x = AiParallel::new(size, Piece::X);
    // let mut b = Board::new(size);
    // ai_x.make_move(&mut b);
}

#[allow(dead_code)]
fn play() {
    let board_size = game::get_board_size();

    // let mut p1 = Human::new("Dan", Piece::X);
    // let mut p1 = AiSerial::new(board_size, Piece::X, 100);
    let mut p1 = AiParallel::new(board_size, Piece::X);
    // let mut p1 = AiLazy::new(board_size, Piece::X);
    // p1.load_strategy();

    let mut p2 = Human::new("Dan", Piece::O);
    // let mut p2 = AiSerial::new(board_size, Piece::O, 100);
    // let mut p2 = AiParallel::new(board_size, Piece::O);
    // let mut p2 = AiLazy::new(Board_size, Piece::O);
    // p2.load_strategy();

    game::play_game(&mut p1, &mut p2, board_size);
    // p1.save_strategy();
}
