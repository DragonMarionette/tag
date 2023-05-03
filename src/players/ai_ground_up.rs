use ciborium::{de, ser};
use serde::{Serialize, Deserialize};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::iter::empty;
use std::ops::Deref;

use super::{available_spaces, MoveValue, Player};
use crate::space::{Coord, Piece};
use crate::Board;
use crate::ScrambledBoard;

const PIECE_TYPES: [Piece;3] = [Piece::X, Piece::O, Piece::Empty];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveAnalysis {
    pub evaluation: MoveValue,
    pub move_options: Vec<Coord>
}

pub struct AiGroundUp {
    size: usize,
    piece: Piece,
    known_boards: HashMap<Board, MoveAnalysis>,
}

impl Display for AiGroundUp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.piece.colorize("AI"), self.piece)
    }
}

impl Player for AiGroundUp {
    fn make_move(&mut self, game_board: &mut Board) {
        assert_eq!(game_board.size, self.size);

        let chosen_move = self.choose_move(self.piece, game_board);
        game_board
            .place(self.piece(), chosen_move.row, chosen_move.col)
            .unwrap();
    }

    fn piece(&self) -> Piece {
        self.piece
    }
}

impl AiGroundUp {
    pub fn new(size: usize, piece: Piece) -> Self {
        Self {
            size,
            piece,
            known_boards: HashMap::new(),
        }
    }

    fn choose_move(&mut self, piece_to_play: Piece, game_board: &Board) -> Coord {
        let mut scrambled = ScrambledBoard::from(game_board.clone());
        if piece_to_play != self.piece {
            // if asking to play a different piece than known_boards assumes
            scrambled.invert();
        }
        scrambled.standardize();

        let key = Board::from(&scrambled);

        let analysis = self.analyze(&key);

        let chosen_move = analysis
            .move_options
            .choose(&mut rand::thread_rng())
            .unwrap();
        scrambled.space_at(chosen_move.clone()).unwrap().to_coord()
    }

    fn analyze(&mut self, key: &Board) -> MoveAnalysis {
        if let Some(analysis) = self.known_boards.get(key) {
            analysis.clone()
        } else {
            self.build_strategy();
            self.analyze(key)
        }
    }

    fn build_strategy(&mut self) {
        /*
        for all possible full, sorted boards {
            set analysis value to either tie or win
            insert analysis
        }
        for all possible move # in (board.size**2-2)..=0 {
            for all sorted boards with that # of moves {
                if has win {
                    insert analysis
                } else {
                    analyses = for each available move {
                        sort
                        value = (value inverted) + 1
                        (move, value)
                    }.filter(Some).collect()
                    max_value = analyses.map(value).max()
                    move_options = analyses.filter_map(value == max => Some(move)).collect()
                    insert analysis
                }
            }
        }
        */
    }

    pub fn cbor_path(&self, inverted: bool) -> String {
        let p = if inverted {
            self.piece.inverse()
        } else {
            self.piece
        };

        let piece_str = match p {
            Piece::X => "X",
            Piece::O => "O",
            _ => "_",
        };
        format!(
            "strategies/ground-up-s{}-p{}.cbor",
            self.size, piece_str
        )
    }

    pub fn save_strategy(&self) {
        let buffer = File::create(&self.cbor_path(false)).unwrap(); // TODO: make safe
        ser::into_writer(&self.known_boards, buffer).unwrap();
        println!("Saved strategy to {}", self.cbor_path(false));
    }

    pub fn load_strategy(&mut self) -> Option<()> {
        if let Ok(f) = File::open(self.cbor_path(false)) {
            self.known_boards = de::from_reader(f).unwrap();
            println!("Read strategy from {}", self.cbor_path(false));
            Some(())
        } else if let Ok(f) = File::open(self.cbor_path(true)) {
            let known_boards_inverted: HashMap<Board, MoveAnalysis> = de::from_reader(f).unwrap();
            for b in known_boards_inverted.keys() {
                let analysis = known_boards_inverted.get(b).unwrap();
                self.known_boards.insert(b.inverse(), analysis.clone());
            }
            println!("Read strategy from {}", self.cbor_path(true));
            Some(())
        } else {
            None
        }
    }
}

// struct BoardIterator<'a> {
//     remaining_piece_counts: HashMap<Piece, u8>,
//     piece_type_iterator: core::slice::Iter<'a, Piece>,
//     current_piece: Piece,
//     lower_iterator: Box<BoardIterator<'a>>,
// }

// impl <'a> Iterator for BoardIterator<'a> {
//     type Item = Vec<Piece>;
//     fn next(&mut self) -> Option<Self::Item> {
//         let mut tail: Vec<Piece>;
//         loop {
//             if let Some(tail_from_lower) = self.lower_iterator.next() {
//                 tail = tail_from_lower;
//                 break;
//             }

//             // lower_iter exhausted
//             // so, cycle self.current_piece until you hit one with a nonzero count
//             while self.remaining_piece_counts.get(&self.current_piece).unwrap() == &0 {
//                 if let Some(current_piece) = self.piece_type_iterator.next() {
//                     self.current_piece = *current_piece;
//                     let mut lower_counts = self.remaining_piece_counts.clone();
//                     *lower_counts.get_mut(&self.current_piece).unwrap() -= 1;

//                     self.lower_iterator = Box::new(
//                         BoardIterator::with_counts(lower_counts)
//                     );
//                 } else {
//                     return None;
//                 }
//             }
//         }

//         // Now we have a valid current piece and a valid tail 
//         let mut vec_out = vec![self.current_piece];
//         vec_out.append(&mut tail);
//         return Some(vec_out);
//     }
// }

// impl <'a> BoardIterator<'a> {
//     // creates iterator over all filled boards
//     fn new(board_size: usize, starting_piece: Piece) -> Self {
//         let board_size = board_size as u8;
//         assert_ne!(starting_piece, Piece::Empty);

//         let mut remaining_piece_counts = HashMap::new();
//         remaining_piece_counts.insert(starting_piece, (board_size*board_size + 1) / 2);
//         remaining_piece_counts.insert(starting_piece.inverse(), board_size*board_size / 2);

//         let mut piece_type_iterator = PIECE_TYPES.iter();
//         let current_piece = piece_type_iterator.next().copied().unwrap();
//         Self {
//             remaining_piece_counts,
//             piece_type_iterator,
//             current_piece,
//             lower_iterator,
//         }
//     }

//     fn with_counts(remaining_piece_counts: HashMap<Piece, u8>) -> Self {
//         let mut piece_type_iterator = PIECE_TYPES.iter();
//         let current_piece = piece_type_iterator.next().copied().unwrap();

//         Self {
//             remaining_piece_counts,
//             piece_type_iterator,
//             current_piece,
//             lower_iterator,
//         }
//     }
// }

fn iterate_boards(board_size: usize, x_count: u8, o_count: u8, empty_count: u8) -> AnyIter<Vec<Piece>>{
    let iter_x = match x_count {
        0 => AnyIter::new(empty()),
        _ => AnyIter::new(
            iterate_boards(board_size, x_count - 1, o_count, empty_count).map(|mut v|{
                let mut vec_out = vec![Piece::X];
                vec_out.append(&mut v);
                vec_out
            }
        ))
    };
    
    let iter_o = match x_count {
        0 => AnyIter::new(empty()),
        _ => AnyIter::new(
            iterate_boards(board_size, x_count, o_count - 1, empty_count).map(|mut v|{
                let mut vec_out = vec![Piece::O];
                vec_out.append(&mut v);
                vec_out
            }
        ))
    };

    let iter_empty = match x_count {
        0 => AnyIter::new(empty()),
        _ => AnyIter::new(
            iterate_boards(board_size, x_count, o_count, empty_count - 1).map(|mut v|{
                let mut vec_out = vec![Piece::Empty];
                vec_out.append(&mut v);
                vec_out
            }
        ))
    };

    AnyIter::new(iter_x.chain(iter_o).chain(iter_empty))
}

struct AnyIter<T> {
    inner: Box<dyn Iterator<Item=T>>
}

impl <T> Iterator for AnyIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T> AnyIter<T> {
    fn new(inner: impl Iterator<Item=T>) -> AnyIter<T>{
        AnyIter {
            inner: Box::new(inner)
        }
    }
}

impl<T> Deref for AnyIter<T> {
    type Target = &dyn Iterator<T>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}