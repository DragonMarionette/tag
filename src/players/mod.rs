use std::cmp::Ordering;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::scrambled_board::Coord;
use crate::{Board, Piece};


mod human;
pub use human::Human;

mod ai_serial;
pub use ai_serial::AiSerial;

mod ai_pruned;
pub use ai_pruned::AiPruned;

mod ai_parallel;
pub use ai_parallel::AiParallel;

pub trait Player: Display {
    fn make_move(&mut self, game_board: &mut Board);
    fn piece(&self) -> Piece;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveAnalysis {
    pub evaluation: MoveValue,
    pub move_options: Vec<Coord>,
    pub depth_used: usize,
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