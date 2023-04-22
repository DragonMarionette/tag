use crate::{
    players::Player,
    space::Piece,
    Board,
};

#[derive(Clone, Copy)]
pub enum Winner {
    P1,
    P2,
    Tie,
}

pub fn play_game(p1: &mut Box<dyn Player>, p2: &mut Box<dyn Player>, board_size: usize) -> Winner {
    assert_ne!(p1.piece(), Piece::Empty);
    assert_ne!(p2.piece(), Piece::Empty);
    assert_ne!(p1.piece(), p2.piece());

    let mut game_board = Board::new(board_size);
    println!("\n{}\n\n", game_board.pretty());

    for turn in [Winner::P1, Winner::P2].iter().cycle() {
        if game_board.is_full() {
            return Winner::Tie;
        }

        let current_player = match turn {
            Winner::P1 => &mut *p1,
            Winner::P2 => &mut *p2,
            _ => {
                panic!("turn was neither P1 nor P2, which should never happen");
            }
        };

        current_player.make_move(&mut game_board);

        println!("\n{}\n\n", game_board.pretty());

        if game_board.has_win(current_player.piece()) {
            return *turn;
        }
    }
    panic!("Loop ended, which should never happen")
}

