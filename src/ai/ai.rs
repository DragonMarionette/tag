use std::{collections::HashMap, cmp::Ordering};
use rand::seq::SliceRandom;

use crate::{board::Board, piece::Piece};

use super::scrambled_board::{ScrambledBoard, Coord};


#[derive(Debug, Clone, PartialEq, Eq)]
enum MoveValue {
    Lose(u8),
    Tie(u8),
    Unknown(u8),
    Win(u8) // number of moves
}

impl PartialOrd for MoveValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MoveValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            MoveValue::Lose(v) => {
                match other {
                    MoveValue::Lose(w) => v.cmp(w),
                    _ => Ordering::Less,
                }
            },
            MoveValue::Tie(v) => {
                match other {
                    MoveValue::Lose(_) => Ordering::Greater,
                    MoveValue::Tie(w) => v.cmp(w),
                    _ => Ordering::Less,
                }
            },
            MoveValue::Unknown(v) => {
                match other {
                    MoveValue::Win(_) => Ordering::Less,
                    MoveValue::Unknown(w) => w.cmp(v),
                    _ => Ordering::Greater,
                }
            },
            MoveValue::Win(v) => {
                match other {
                    MoveValue::Win(w) => w.cmp(v),
                    _ => Ordering::Greater,
                }
            },
        }
    }
}


#[derive(Debug, Clone)]
struct MoveAnalysis {
    evaluation: MoveValue,
    move_options: Vec<Coord>,
    depth_used: usize
}

pub struct AI {
    pub piece: Piece,
    depth: usize,
    known_boards: HashMap<Board, MoveAnalysis>,
    rng: rand::rngs::ThreadRng,
}


impl AI {
    pub fn new(piece: Piece, depth: usize) -> AI {
        AI { piece, depth, known_boards: HashMap::new(), rng: rand::thread_rng() }
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

        // let move_options = analysis.move_options;
        let chosen_move = analysis.move_options.choose(&mut self.rng).unwrap();
        Coord::from_space(&scrambled.space_at(chosen_move.clone()).unwrap())
    }

    fn analyze(&mut self, b: &Board, depth_remaining: usize) -> MoveAnalysis { // assumes it is getting an already-standardized board
        if let Some(analysis) = self.known_boards.get(b) { // b already computed to sufficient depth
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

        if depth_remaining == 0 {
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
        for c in available_spaces(b) {
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

            // // short-circuit to never explore after the first known win. Slows things down for some reason
            // if let MoveValue::Win(_) = lower_analysis.evaluation {
            //     let new_analysis = MoveAnalysis {
            //         evaluation: lower_analysis.evaluation,
            //         move_options: vec![c],
            //         depth_used: lower_analysis.depth_used + 1,
            //     };
            //     self.known_boards.insert(b.clone(), new_analysis.clone());
        
            //     return new_analysis;
            // }

            new_analyses.push((c, lower_analysis))
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
        self.known_boards.insert(b.clone(), new_analysis.clone());

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
