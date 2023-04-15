use rand::seq::SliceRandom;
use std::fmt::Display;
use std::{cmp::Ordering, collections::HashMap};

use crate::scrambled_board::{Coord, ScrambledBoard};
use crate::{board::Board, piece::Piece};
use super::Player;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveValue {
    // u8 represents number of moves until the outcome is guaranteed achieveable
    Lose(u8),
    Tie(u8),
    Unknown(u8),
    Win(u8),
}

impl PartialOrd for MoveValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MoveValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            MoveValue::Lose(v) => match other {
                MoveValue::Lose(w) => v.cmp(w),
                _ => Ordering::Less,
            },
            MoveValue::Tie(v) => match other {
                MoveValue::Lose(_) => Ordering::Greater,
                MoveValue::Tie(w) => v.cmp(w),
                _ => Ordering::Less,
            },
            MoveValue::Unknown(v) => match other {
                MoveValue::Win(_) => Ordering::Less,
                MoveValue::Unknown(w) => w.cmp(v),
                _ => Ordering::Greater,
            },
            MoveValue::Win(v) => match other {
                MoveValue::Win(w) => w.cmp(v),
                _ => Ordering::Greater,
            },
        }
    }
}

impl MoveValue {
    #[allow(dead_code)]
    pub fn depth(&self) -> u8 {
        match *self {
            MoveValue::Lose(v) => v,
            MoveValue::Tie(v) => v,
            MoveValue::Unknown(v) => v,
            MoveValue::Win(v) => v,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoveAnalysis {
    pub evaluation: MoveValue,
    pub move_options: Vec<Coord>,
    pub depth_used: usize,
}

pub struct AiSerial {
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
    pub fn new(piece: Piece, depth: usize) -> Self {
        Self {
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
        Coord::from_space(&scrambled.space_at(chosen_move.clone()).unwrap())
    }

    fn analyze(&mut self, b: &Board, depth_remaining: usize) -> MoveAnalysis {
        // assumes it is getting an already-standardized board
        if let Some(analysis) = self.known_boards.get(b) {
            // b already computed to sufficient depth
            if analysis.depth_used >= depth_remaining {
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
        available_spaces(b).into_iter().for_each(|c| {
            let mut b = b.clone();
            b.place(self.piece, c.row, c.col).unwrap();
            b.invert();
            let mut scrambled = ScrambledBoard::from_board(&b);
            scrambled.standardize();
            let mut lower_analysis = self.analyze(&scrambled.to_board(), depth_remaining - 1);

            lower_analysis.evaluation = match lower_analysis.evaluation {
                MoveValue::Lose(v) => MoveValue::Win(v + 1),
                MoveValue::Tie(v) => MoveValue::Tie(v + 1),
                MoveValue::Unknown(v) => MoveValue::Unknown(v + 1),
                MoveValue::Win(v) => MoveValue::Lose(v + 1),
            };

            new_analyses.push((c, lower_analysis))
        });
        let shallowest_depth = new_analyses.iter().map(|a| a.1.depth_used).min().unwrap();
        let depth_used = shallowest_depth + 1;

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
                result.push(Coord { row, col })
            }
        }
    }
    result
}
