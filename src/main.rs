#[allow(unused_imports)]
use tag::{
    game,
    players::{AiLazy, AiParallel, AiSerial, Human, Player},
    Board, space::Piece,
};

fn main() {
    play();
}

#[allow(dead_code)]
fn play() {
    let board_size = game::get_board_size();

    let mut p1 = AiParallel::new(board_size, Piece::X, false);

    let mut p2 = Human::new("Dan", Piece::O);

    game::play_game(&mut p1, &mut p2, board_size);
}
