use ciborium::{de, ser};
use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;

use rayon::prelude::*;
use std::sync::{Arc, RwLock};

use super::{available_spaces, MoveAnalysis, MoveValue, Player};
use crate::space::{Coord, Piece};
use crate::Board;
use crate::ScrambledBoard;

const MAX_SERIAL_DEPTH: usize = 3; // magic value found experimentally
const MAX_DEPTH: usize = 100;

pub struct AiParallel {
    board_size: usize,
    piece: Piece,
    known_boards: Arc<RwLock<HashMap<Board, MoveAnalysis>>>,
}

impl Display for AiParallel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.piece.colorize("AI"), self.piece)
    }
}

impl Player for AiParallel {
    fn make_move(&mut self, game_board: &mut Board) {
        let chosen_move = self.choose_move(self.piece, game_board);
        game_board.place(self.piece(), chosen_move).unwrap();
    }

    fn piece(&self) -> Piece {
        self.piece
    }
}

impl AiParallel {
    pub fn new(size: usize, piece: Piece) -> Self {
        Self {
            board_size: size,
            piece,
            known_boards: Arc::new(RwLock::new(HashMap::new())),
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

        let analysis = self.analyze(&key, 0, &Vec::new());

        let mut chosen_move_initial = *analysis
            .move_options
            .choose(&mut rand::thread_rng())
            .unwrap();
        chosen_move_initial = scrambled.space_at(chosen_move_initial).unwrap().coord;

        self.equivalent_move(chosen_move_initial, game_board)
    }

    fn equivalent_move(&self, reference_coord: Coord, b: &Board) -> Coord {
        let mut moves: HashMap<Coord, Board> = HashMap::new();

        for c in available_spaces(b) {
            let mut this_board = b.clone();
            if this_board.place(self.piece, c).is_ok() {
                this_board.fully_standardize();
                moves.insert(c, this_board);
            }
        }

        let mut equivalent: Vec<Coord> = Vec::new();
        let reference_board = moves.get(&reference_coord).unwrap().clone();

        for (c, compared_board) in moves {
            if compared_board == reference_board {
                equivalent.push(c);
            }
        }
        *equivalent.choose(&mut thread_rng()).unwrap()
    }

    fn analyze(
        &self,
        b: &Board,
        current_depth: usize,
        parents: &Vec<Arc<RwLock<bool>>>,
    ) -> MoveAnalysis {
        // assumes it is getting an already-standardized board
        if let Some(analysis) = self.known_boards.read().unwrap().get(b) {
            // b already computed to sufficient depth
            return analysis.clone();
        }

        if b.has_win(self.piece.inverse()) {
            // b already has other player winning
            let new_analysis = MoveAnalysis {
                evaluation: MoveValue::Lose(0),
                move_options: vec![],
                depth_used: MAX_DEPTH, // max depth because no need to ever reanalyze this position deeper
            };
            self.known_boards
                .write()
                .unwrap()
                .insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        if b.is_full() {
            let new_analysis = MoveAnalysis {
                evaluation: MoveValue::Tie(0),
                move_options: vec![],
                depth_used: MAX_DEPTH, // max depth because no need to ever reanalyze this position deeper
            };
            self.known_boards
                .write()
                .unwrap()
                .insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        // recursive case
        let win_found = RwLock::new(false);
        let mut new_analyses: Vec<(Coord, MoveAnalysis)>;
        new_analyses = if current_depth <= MAX_SERIAL_DEPTH {
            // serial
            available_spaces(b)
                .into_iter()
                .map(|c| {
                    if *win_found.read().unwrap() || parents.iter().any(|rw| *rw.read().unwrap()) {
                        return None;
                    }
                    let mut recursion_board = b.clone();
                    recursion_board.place(self.piece, c).unwrap();
                    recursion_board.invert();
                    recursion_board.standardize();

                    let mut parents_inner = parents.clone();
                    parents_inner.push(Arc::new(RwLock::new(false)));

                    let mut lower_analysis =
                        self.analyze(&recursion_board, current_depth + 1, &parents_inner);

                    lower_analysis.evaluation = lower_analysis.evaluation.invert();
                    lower_analysis.evaluation = lower_analysis.evaluation.increment();

                    if let MoveValue::Win(_) = lower_analysis.evaluation {
                        *parents_inner.last().unwrap().write().unwrap() = true;
                        *win_found.write().unwrap() = true;
                    }

                    Some((c, lower_analysis))
                })
                .map_while(|a| a)
                .collect()
        } else {
            // parallel
            available_spaces(b)
                .into_par_iter()
                .filter_map(|c| {
                    if *win_found.read().unwrap() || parents.iter().any(|rw| *rw.read().unwrap()) {
                        return None;
                    }

                    let mut recursion_board: Board = b.clone();
                    recursion_board.place(self.piece, c).unwrap();
                    recursion_board.invert();
                    recursion_board.standardize();

                    let mut parents_inner = parents.clone();
                    parents_inner.push(Arc::new(RwLock::new(false)));

                    let mut lower_analysis =
                        self.analyze(&recursion_board, current_depth + 1, &parents_inner);

                    lower_analysis.evaluation = lower_analysis.evaluation.invert();
                    lower_analysis.evaluation = lower_analysis.evaluation.increment();

                    if let MoveValue::Win(_) = lower_analysis.evaluation {
                        *parents_inner.last().unwrap().write().unwrap() = true;
                        *win_found.write().unwrap() = true;
                    }

                    Some((c, lower_analysis))
                })
                .collect()
        }; // parallel

        if new_analyses.is_empty() {
            // short circuited by another thread
            let new_analysis = MoveAnalysis {
                evaluation: MoveValue::Unknown(0),
                move_options: vec![],
                depth_used: 0,
            };

            return new_analysis;
        }

        let shallowest_depth = new_analyses.iter().map(|a| a.1.depth_used).min().unwrap();
        let depth_used = shallowest_depth + 1;

        // filter to keep only the best-evaluated moves
        let best_evaluation = new_analyses
            .iter()
            .map(|a| a.1.evaluation.clone())
            .max()
            .unwrap();
        new_analyses.retain(|a| a.1.evaluation == best_evaluation);

        let move_options = new_analyses.iter().map(|a| a.0).collect();

        let new_analysis = MoveAnalysis {
            evaluation: best_evaluation,
            move_options,
            depth_used,
        };

        self.known_boards
            .write()
            .unwrap()
            .insert(b.clone(), new_analysis.clone());

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
            "strategies/parallel-s{}-p{}-lazy.cbor",
            self.board_size, piece_str
        )
    }

    pub fn save_strategy(&self) {
        let buffer = File::create(self.cbor_path(false)).unwrap(); // TODO: make safe
        ser::into_writer(&*self.known_boards.read().unwrap(), buffer).unwrap();
        println!("Saved strategy to {}", self.cbor_path(false));
    }

    pub fn load_strategy(&mut self) -> Option<()> {
        if let Ok(f) = File::open(self.cbor_path(false)) {
            *self.known_boards.write().unwrap() = de::from_reader(f).unwrap();
            println!("Read strategy from {}", self.cbor_path(false));
            Some(())
        } else if let Ok(f) = File::open(self.cbor_path(true)) {
            let known_boards_inverted: HashMap<Board, MoveAnalysis> = de::from_reader(f).unwrap();
            for b in known_boards_inverted.keys() {
                let analysis = known_boards_inverted.get(b).unwrap();
                self.known_boards
                    .write()
                    .unwrap()
                    .insert(b.inverse(), analysis.clone());
            }
            println!("Read strategy from {}", self.cbor_path(true));
            Some(())
        } else {
            None
        }
    }
}
