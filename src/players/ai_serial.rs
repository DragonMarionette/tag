use ciborium::{de, ser};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;

use super::{available_spaces, MoveAnalysis, MoveValue, Player};
use crate::space::{Coord, Piece};
use crate::Board;
use crate::ScrambledBoard;

pub struct AiSerial {
    size: usize,
    piece: Piece,
    depth: usize,
    known_boards: HashMap<Board, MoveAnalysis>,
}

impl Display for AiSerial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.piece.colorize("AI"), self.piece)
    }
}

impl Player for AiSerial {
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

impl AiSerial {
    pub fn new(size: usize, piece: Piece, depth: usize) -> Self {
        let depth = if depth > size*size {
            size*size
        } else {
            depth
        };
        
        Self {
            size,
            piece,
            depth,
            known_boards: HashMap::new(),
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

        let analysis = self.analyze(&key, self.depth);

        let chosen_move = analysis
            .move_options
            .choose(&mut rand::thread_rng())
            .unwrap();
        scrambled.space_at(chosen_move.clone()).unwrap().to_coord()
    }

    fn analyze(&mut self, b: &Board, depth_to_use: usize) -> MoveAnalysis {
        // assumes it is getting an already-standardized board
        if let Some(analysis) = self.known_boards.get(b) {
            // b already computed to sufficient depth
            if analysis.depth_used >= depth_to_use {
                return analysis.clone();
            }
        }

        if b.has_win(self.piece.inverse()) {
            // b already has other player winning
            let new_analysis = MoveAnalysis {
                evaluation: MoveValue::Lose(0),
                move_options: vec![],
                depth_used: self.depth, // max depth because no need to ever reanalyze this position deeper
            };
            self.known_boards.insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        if b.is_full() {
            let new_analysis = MoveAnalysis {
                evaluation: MoveValue::Tie(0),
                move_options: vec![],
                depth_used: self.depth, // max depth because no need to ever reanalyze this position deeper
            };
            self.known_boards.insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        if depth_to_use == 0 {
            let new_analysis = MoveAnalysis {
                evaluation: MoveValue::Unknown(0),
                move_options: available_spaces(b),
                depth_used: 0,
            };
            self.known_boards.insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        // recursive case
        let mut new_analyses: Vec<(Coord, MoveAnalysis)> = Vec::new();
        available_spaces(b).into_iter().for_each(|c| {
            let mut b = b.clone();
            b.place(self.piece, c.row, c.col).unwrap();
            b.invert();
            let mut scrambled = ScrambledBoard::from_board(&b);
            scrambled.standardize();
            let mut lower_analysis = self.analyze(&scrambled.to_board(), depth_to_use - 1);

            lower_analysis.evaluation = match lower_analysis.evaluation {
                MoveValue::Lose(v) => MoveValue::Win(v + 1),
                MoveValue::Tie(v) => MoveValue::Tie(v + 1),
                MoveValue::Unknown(v) => MoveValue::Unknown(v + 1),
                MoveValue::Win(v) => MoveValue::Lose(v + 1),
            };

            new_analyses.push((c, lower_analysis));
        });
        let shallowest_depth = new_analyses.iter().map(|a| a.1.depth_used).min().unwrap();
        let depth_used = shallowest_depth + 1;

        // filter to keep only the best-evaluated moves.TODO: simplify with an accumulator?
        let best_evaluation = new_analyses
            .iter()
            .map(|a| a.1.evaluation.clone())
            .max()
            .unwrap();
        new_analyses = new_analyses
            .into_iter()
            .filter(|a| a.1.evaluation == best_evaluation)
            .collect();

        let move_options = new_analyses.iter().map(|a| a.0).collect();

        let new_analysis = MoveAnalysis {
            evaluation: best_evaluation,
            move_options,
            depth_used,
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
        format!("strategies/serial-s{}-p{}-d{}.cbor", self.size, piece_str, self.depth)
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
