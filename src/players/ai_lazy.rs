use ciborium::{de, ser};
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;

use super::{available_spaces_shuffled, MoveAnalysis, MoveValue, Player};
use crate::space::{Coord, Piece};
use crate::Board;
use crate::ScrambledBoard;
// use crate::scrambled_board_flat::ScrambledBoard;

const MAX_DEPTH: usize = 100;

pub struct AiLazy {
    size: usize,
    piece: Piece,
    known_boards: HashMap<Board, MoveAnalysis>,
    rng: SmallRng,
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
    pub fn new(size: usize, piece: Piece) -> Self {
        Self {
            size,
            piece,
            known_boards: HashMap::new(),
            rng: SmallRng::from_entropy(),
        }
    }

    fn choose_move(&mut self, player: Piece, game_board: &Board) -> Coord {
        let mut scrambled = ScrambledBoard::from_board(game_board);
        if player != self.piece {
            // if asking to play a different piece than known_boards assumes
            scrambled.invert();
        }
        scrambled.standardize();

        let key = scrambled.to_board();

        let analysis = self.analyze(&key);

        let chosen_move = analysis
            .move_options
            .choose(&mut rand::thread_rng())
            .unwrap();
        scrambled.space_at(chosen_move.clone()).unwrap().to_coord()
    }

    fn analyze(&mut self, b: &Board) -> MoveAnalysis {
        // assumes it is getting an already-standardized board
        if let Some(analysis) = self.known_boards.get(b) {
            return analysis.clone();
        }

        if b.has_win(self.piece.inverse()) {
            // b already has other player winning
            let new_analysis = MoveAnalysis {
                evaluation: MoveValue::Lose(0),
                move_options: vec![],
                depth_used: MAX_DEPTH, // no need to ever reanalyze this position deeper
            };
            self.known_boards.insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        if b.is_full() {
            let new_analysis = MoveAnalysis {
                evaluation: MoveValue::Tie(0),
                move_options: vec![],
                depth_used: MAX_DEPTH, // no need to ever reanalyze this position deeper
            };
            self.known_boards.insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        // recursive case
        let mut new_analyses: Vec<(Coord, MoveAnalysis)> = Vec::new();
        for c in available_spaces_shuffled(b, &mut self.rng) {
            let mut recursion_board = b.clone();
            recursion_board.place(self.piece, c.row, c.col).unwrap();
            recursion_board.invert();
            let mut scrambled = ScrambledBoard::from_board(&recursion_board);
            scrambled.standardize();
            let mut lower_analysis = self.analyze(&scrambled.to_board());

            lower_analysis.evaluation = match lower_analysis.evaluation {
                MoveValue::Lose(v) => MoveValue::Win(v + 1),
                MoveValue::Tie(v) => MoveValue::Tie(v + 1),
                MoveValue::Unknown(v) => MoveValue::Unknown(v + 1),
                MoveValue::Win(v) => MoveValue::Lose(v + 1),
            };

            // short circuit as soon as a win is found. Thus considers every win equally optimal
            if let MoveValue::Win(_) = lower_analysis.evaluation {
                let new_analysis = MoveAnalysis {
                    evaluation: lower_analysis.evaluation,
                    move_options: vec![c],
                    depth_used: MAX_DEPTH,
                };

                self.known_boards.insert(b.clone(), new_analysis.clone());
                return new_analysis;
            }

            new_analyses.push((c, lower_analysis))
        }

        // filter to keep only the best-evaluated moves
        let best_evaluation = new_analyses
            .iter()
            .map(|a| a.1.evaluation.clone())
            .max()
            .unwrap();
        new_analyses = new_analyses
            .iter()
            .filter_map(|a| {
                if a.1.evaluation == best_evaluation {
                    Some(a.clone())
                } else {
                    None
                }
            })
            .collect();

        let move_options = new_analyses.iter().map(|a| a.0).collect();

        let new_analysis = MoveAnalysis {
            evaluation: best_evaluation,
            move_options,
            depth_used: MAX_DEPTH,
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
        format!(
            "strategies/known-moves-s{}-p{}-lazy.cbor",
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
