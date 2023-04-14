use std::sync::{Arc, Mutex};
use std::{collections::HashMap, cmp::Ordering};
use rand::seq::SliceRandom;
use rayon::prelude::*;

use crate::{board::Board, piece::Piece};

use super::scrambled_board::{ScrambledBoard, Coord};
use super::ai::{MoveAnalysis, MoveValue};

pub struct AiMulti {
    pub piece: Piece,
    depth: usize,
    known_boards: Arc<Mutex<HashMap<Board, MoveAnalysis>>>
}


impl AiMulti {
    pub fn new(piece: Piece, depth: usize) -> AiMulti {
        AiMulti { piece, depth, known_boards: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn make_move(&mut self, player: Piece, game_board: &mut Board) {
        let chosen_move = self.choose_move(player, game_board);
        game_board.place(player, chosen_move.row, chosen_move.col).unwrap();
    }

    fn choose_move(&mut self, player: Piece, game_board: &Board) -> Coord{
        let mut scrambled = ScrambledBoard::from_board(game_board);
        if player != self.piece { // if asking to play a different piece than known_boards assumes
            scrambled.invert();
        }
        scrambled.standardize();

        let key = scrambled.to_board();

        let analysis = self.analyze(&key, self.depth);

        let chosen_move = analysis.move_options.choose(&mut rand::thread_rng()).unwrap();
        Coord::from_space(&scrambled.space_at(chosen_move.clone()).unwrap())
    }

    fn analyze(&self, b: &Board, depth_remaining: usize) -> MoveAnalysis { // assumes it is getting an already-standardized board
        if let Some(analysis) = self.known_boards.lock().unwrap().get(b) { // b already computed to sufficient depth
            if analysis.depth_used >= depth_remaining {
                return analysis.clone();
            }
        }

        if b.has_win(self.piece.inverse()) { // b already has other player winning
            let new_analysis = MoveAnalysis {
                evaluation: MoveValue::Lose(0),
                move_options: vec![],
                depth_used: self.depth, // max depth because no need to ever reanalyze this position deeper
            };
            self.known_boards.lock().unwrap().insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        if b.is_full() {
            let new_analysis = MoveAnalysis {
                evaluation: MoveValue::Tie(0),
                move_options: vec![],
                depth_used: self.depth, // max depth because no need to ever reanalyze this position deeper
            };
            self.known_boards.lock().unwrap().insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        if depth_remaining == 0 {
            let new_analysis = MoveAnalysis {
                evaluation: MoveValue::Unknown(0),
                move_options: available_spaces(b),
                depth_used: 0,
            };
            self.known_boards.lock().unwrap().insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        // recursive case
        let mut new_analyses: Vec<(Coord, MoveAnalysis)>;
        if self.depth - depth_remaining <= 2 { // series. 2 is a magic number found experimentally
            new_analyses = Vec::new();
            available_spaces(b).into_iter().for_each(|c| {
                let mut b = b.clone();
                b.place(self.piece, c.row, c.col).unwrap();
                b.invert();
                let mut scrambled = ScrambledBoard::from_board(&b);
                scrambled.standardize();
                let mut lower_analysis = self.analyze(&scrambled.to_board(), depth_remaining-1);
    
                lower_analysis.evaluation = match lower_analysis.evaluation {
                    MoveValue::Lose(v) => MoveValue::Win(v+1),
                    MoveValue::Tie(v) => MoveValue::Tie(v+1),
                    MoveValue::Unknown(v) => MoveValue::Unknown(v+1),
                    MoveValue::Win(v) => MoveValue::Lose(v+1),
                };
    
                new_analyses.push((c, lower_analysis))
            });
        } else { // parallel
            let new_analyses_shared = Arc::new(Mutex::new(Vec::new()));
            available_spaces(b).into_par_iter().for_each(|c| {
                let mut b = b.clone();
                b.place(self.piece, c.row, c.col).unwrap();
                b.invert();
                let mut scrambled = ScrambledBoard::from_board(&b);
                scrambled.standardize();
                let mut lower_analysis = self.analyze(&scrambled.to_board(), depth_remaining-1);
    
                lower_analysis.evaluation = match lower_analysis.evaluation {
                    MoveValue::Lose(v) => MoveValue::Win(v+1),
                    MoveValue::Tie(v) => MoveValue::Tie(v+1),
                    MoveValue::Unknown(v) => MoveValue::Unknown(v+1),
                    MoveValue::Win(v) => MoveValue::Lose(v+1),
                };
    
                new_analyses_shared.lock().unwrap().push((c, lower_analysis))
            });
            new_analyses = new_analyses_shared.lock().unwrap().clone();
        }
        
        let shallowest_depth = new_analyses.iter()
            .map(|a| a.1.depth_used)
            .min().unwrap();
        let depth_used = shallowest_depth + 1;

        // filter to keep only the best-evaluated moves
        let best_evaluation = new_analyses.iter().map(|a| a.1.evaluation.clone())
            .max().unwrap();
        new_analyses = new_analyses.iter()
            .filter_map(|a| if a.1.evaluation == best_evaluation {Some(a.clone())} else {None})
            .collect();

        let move_options = new_analyses.iter()
            .map(|a| a.0)
            .collect();

        let new_analysis = MoveAnalysis {
            evaluation: best_evaluation,
            move_options,
            depth_used,
        };
        self.known_boards.lock().unwrap().insert(b.clone(), new_analysis.clone());

        new_analysis
    }
}

fn available_spaces(b: &Board) -> Vec<Coord> {
    let mut result = Vec::new();
    for row in 0..b.size {
        for col in 0..b.size {
            if b.piece_at(row, col) == Ok(Piece::Empty) {
                result.push(Coord {row, col})
            }
        }
    }
    result
}
