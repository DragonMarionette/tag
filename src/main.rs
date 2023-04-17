#[allow(unused_imports)]
use tag::{
    game,
    Piece,
    Board,
    players::{Player, Human, AiParallel, AiSerial},
};

fn main() {
    play();
    // let size = 4;
    // let mut ai_x = AiSerial::new(size, Piece::X, 100);
    // let mut b = Board::new(size);
    // ai_x.make_move(&mut b);
}

#[allow(dead_code)]
fn play() {
    let board_size = game::get_board_size();

    // let mut p1 = Human::new("Dan", Piece::X);
    let mut p1 = AiSerial::new(board_size, Piece::X, 100);
    p1.load_strategy();
    // let mut p1 = AiParallel::new(Piece::X, 100);

    let mut p2 = Human::new("Dan", Piece::O);
    // let mut p2 = AiSerial::new(board_size, Piece::O, 100);
    // p2.load_strategy();
    // let mut p2 = AiParallel::new(Piece::O, 100);

    game::play_game(&mut p1, &mut p2, board_size);
    // p1.save_strategy();
}
