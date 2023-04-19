use rand::{thread_rng, seq::SliceRandom};
use std::collections::HashMap;
use std::fmt::Display;

use rayon::prelude::*;
use std::sync::{Arc, RwLock};

use super::{available_spaces, MoveAnalysis, MoveValue, Player};
use crate::space::{Coord, Piece};
use crate::Board;
use crate::ScrambledBoard;

const MAX_SERIAL_DEPTH: usize = 3; // magic value found experimentally
const MAX_DEPTH: usize = 100;

pub struct AiParallel {
    pub piece: Piece,
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
        game_board
            .place(self.piece(), chosen_move.row, chosen_move.col)
            .unwrap();
    }

    fn piece(&self) -> Piece {
        self.piece
    }
}

impl AiParallel {
    pub fn new(piece: Piece) -> Self {
        Self {
            piece,
            known_boards: Arc::new(RwLock::new(HashMap::new())),
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

        let analysis = self.analyze(&key, 0, &Vec::new());

        let mut chosen_move_initial = *analysis
            .move_options
            .choose(&mut rand::thread_rng())
            .unwrap();
        chosen_move_initial = scrambled.space_at(chosen_move_initial).unwrap().to_coord();

        let chosen_move = self.equivalent_move(chosen_move_initial, &game_board);

        chosen_move
    }

    fn equivalent_move(&self, reference_coord: Coord, b: &Board) -> Coord {
        let mut moves: HashMap<(usize, usize), Board> = HashMap::new();

        for row in 0..b.size {
            for col in 0..b.size{
                let mut this_board = b.clone();
                if this_board.place(self.piece, row, col).is_ok(){
                    let standardized = ScrambledBoard::from_board(&this_board).into_standardized().to_board();
                    moves.insert((row, col), standardized);
                }
            }
        }
        let mut equivalent: Vec<Coord> = Vec::new();
        let reference_board = moves.get(&(reference_coord.row, reference_coord.col)).unwrap().clone();

        for ((row, col), compared_board) in moves {
            if compared_board == reference_board {
                equivalent.push(Coord { row, col});
            }
        }
        equivalent.choose(&mut thread_rng()).unwrap().clone()
    }

    fn analyze(&self, b: &Board, current_depth: usize, parents: &Vec<Arc<RwLock<bool>>>) -> MoveAnalysis {
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
                    let mut b = b.clone();
                    b.place(self.piece, c.row, c.col).unwrap();
                    b.invert();
                    let mut scrambled = ScrambledBoard::from_board(&b);
                    scrambled.standardize();

                    let mut parents_inner = parents.clone();
                    parents_inner.push(Arc::new(RwLock::new(false)));

                    let mut lower_analysis = self.analyze(
                        &scrambled.to_board(),
                        current_depth + 1,
                        &parents_inner,
                    );

                    lower_analysis.evaluation = match lower_analysis.evaluation {
                        MoveValue::Lose(v) => MoveValue::Win(v + 1),
                        MoveValue::Tie(v) => MoveValue::Tie(v + 1),
                        MoveValue::Unknown(v) => MoveValue::Unknown(v + 1),
                        MoveValue::Win(v) => MoveValue::Lose(v + 1),
                    };

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

                    let mut b = b.clone();
                    b.place(self.piece, c.row, c.col).unwrap();
                    b.invert();
                    let mut scrambled = ScrambledBoard::from_board(&b);
                    scrambled.standardize();

                    let mut parents_inner = parents.clone();
                    parents_inner.push(Arc::new(RwLock::new(false)));

                    let mut lower_analysis = self.analyze(
                        &scrambled.to_board(),
                        current_depth + 1,
                        &parents_inner,
                    );

                    lower_analysis.evaluation = match lower_analysis.evaluation {
                        MoveValue::Lose(v) => MoveValue::Win(v + 1),
                        MoveValue::Tie(v) => MoveValue::Tie(v + 1),
                        MoveValue::Unknown(v) => MoveValue::Unknown(v + 1),
                        MoveValue::Win(v) => MoveValue::Lose(v + 1),
                    };

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

        self.known_boards
            .write()
            .unwrap()
            .insert(b.clone(), new_analysis.clone());

        new_analysis
    }
}
