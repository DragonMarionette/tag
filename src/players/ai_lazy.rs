use ciborium::{de, ser};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;

use super::available_spaces;
use super::{MoveValue, Player};
use crate::space::{Coord, Piece};
use crate::Board;
use crate::ScrambledBoard;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyMoveAnalysis {
    pub evaluation: MoveValue,
    pub move_option: Option<Coord>,
}

pub struct AiLazy {
    size: usize,
    piece: Piece,
    known_boards: HashMap<Board, LazyMoveAnalysis>,
    deterministic: bool,
}

impl Display for AiLazy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.piece.colorize("AI"), self.piece)
    }
}

impl Player for AiLazy {
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

impl AiLazy {
    pub fn new(size: usize, piece: Piece, deterministic: bool) -> Self {
        Self {
            size,
            piece,
            known_boards: HashMap::new(),
            deterministic,
        }
    }

    fn choose_move(&mut self, piece_to_play: Piece, game_board: &Board) -> Coord {
        let mut scrambled = ScrambledBoard::from(game_board.clone());
        if piece_to_play != self.piece {
            scrambled.invert();
        }
        scrambled.standardize();

        let key = Board::from(&scrambled);

        let analysis = self.analyze(&key);

        let mut chosen_move = analysis.move_option.unwrap();
        chosen_move = scrambled.space_at(chosen_move).unwrap().to_coord();

        match self.deterministic {
            true => chosen_move,
            false => self.equivalent_move(chosen_move, &game_board),
        }
    }

    fn equivalent_move(&self, reference_coord: Coord, b: &Board) -> Coord {
        let mut moves: HashMap<(usize, usize), Board> = HashMap::new();

        for row in 0..b.size {
            for col in 0..b.size {
                let mut this_board = b.clone();
                if this_board.place(self.piece, row, col).is_ok() {
                    let standardized =
                        Board::from(ScrambledBoard::from(this_board).into_standardized());
                    moves.insert((row, col), standardized);
                }
            }
        }
        let mut equivalent: Vec<Coord> = Vec::new();
        let reference_board = moves
            .get(&(reference_coord.row, reference_coord.col))
            .unwrap()
            .clone();

        for ((row, col), compared_board) in moves {
            if compared_board == reference_board {
                equivalent.push(Coord { row, col });
            }
        }
        equivalent.choose(&mut thread_rng()).unwrap().clone()
    }

    fn analyze(&mut self, b: &Board) -> LazyMoveAnalysis {
        // assumes it is getting an already-standardized board
        if let Some(analysis) = self.known_boards.get(b) {
            return analysis.clone();
        }

        if b.has_win(self.piece.inverse()) {
            // b already has other player winning
            let new_analysis = LazyMoveAnalysis {
                evaluation: MoveValue::Lose(0),
                move_option: None,
            };
            self.known_boards.insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        if b.is_full() {
            let new_analysis = LazyMoveAnalysis {
                evaluation: MoveValue::Tie(0),
                move_option: None,
            };
            self.known_boards.insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        // recursive case
        let mut best_coord = *available_spaces(b).get(0).unwrap();
        let mut best_evaluation = MoveValue::Lose(0); // lowest possible evaluation to start
        for c in available_spaces(b) {
            let mut recursion_board = b.clone();
            recursion_board.place(self.piece, c.row, c.col).unwrap();
            recursion_board.invert();
            let mut scrambled = ScrambledBoard::from(recursion_board);
            scrambled.standardize();
            let mut lower_analysis = self.analyze(&Board::from(scrambled));

            lower_analysis.evaluation = match lower_analysis.evaluation {
                MoveValue::Lose(v) => MoveValue::Win(v + 1),
                MoveValue::Tie(v) => MoveValue::Tie(v + 1),
                MoveValue::Unknown(v) => MoveValue::Unknown(v + 1),
                MoveValue::Win(v) => MoveValue::Lose(v + 1),
            };

            // short circuit as soon as a win is found. Thus considers every win equally optimal
            if let MoveValue::Win(_) = lower_analysis.evaluation {
                let new_analysis = LazyMoveAnalysis {
                    evaluation: lower_analysis.evaluation,
                    move_option: Some(c),
                };

                self.known_boards.insert(b.clone(), new_analysis.clone());
                return new_analysis;
            }

            if lower_analysis.evaluation > best_evaluation {
                best_coord = c;
                best_evaluation = lower_analysis.evaluation;
            }
        }

        let new_analysis = LazyMoveAnalysis {
            evaluation: best_evaluation,
            move_option: Some(best_coord),
        };
        self.known_boards.insert(b.clone(), new_analysis.clone());

        new_analysis
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
        format!("strategies/lazy-s{}-p{}-lazy.cbor", self.size, piece_str)
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
            let known_boards_inverted: HashMap<Board, LazyMoveAnalysis> =
                de::from_reader(f).unwrap();
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
