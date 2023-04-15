#[allow(unused_imports)]
use tag::{
    game,
    Piece,
    Board,
    players::{Player, Human, AiParallel, AiSerial},
};

fn main() {
    // play();
    let mut ai_x = AiSerial::new(Piece::X, 100);
    let mut b = Board::new(4);
    ai_x.make_move(&mut b);
}

#[allow(dead_code)]
fn play() {
    let board_size = game::get_board_size();

    let mut p1 = Human::new("Dan X", Piece::X);
    // let mut p1 = AiSerial::new(Piece::X, 100);
    // let mut p1 = AiParallel::new(Piece::X, 100);

    // let mut p2 = Human::new("Dan O", Piece::O);
    let mut p2 = AiSerial::new(Piece::O, 100);
    // let mut p2 = AiParallel::new(Piece::O, 100);

    game::play_game(&mut p1, &mut p2, board_size)
}
