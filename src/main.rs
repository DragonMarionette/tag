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

    let mut p1 = game::get_player(Piece::X, board_size);
    let mut p2 = game::get_player(Piece::O, board_size);

    match game::play_game(&mut p1, &mut p2, board_size) {
        game::Winner::P1 => {
            println!("{} wins!", p1)
        },
        game::Winner::P2 => {
            println!("{} wins!", p2)
        }
        game::Winner::Tie => {
            println!("It's a tie!")
        }
    }
}
