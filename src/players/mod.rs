use rand::seq::SliceRandom;
use std::cmp::Ordering;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    space::{Coord, Piece},
    Board,
};

mod human;
pub use human::Human;

mod ai_serial;
pub use ai_serial::AiSerial;

mod ai_lazy;
pub use ai_lazy::AiLazy;

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

fn available_spaces_shuffled(b: &Board, rng: &mut impl rand::Rng) -> Vec<Coord> {
    let mut vec_out = available_spaces(b);
    vec_out.shuffle(rng);
    vec_out
}

use rand_core::{RngCore, Error, impls};
use rand::SeedableRng;
use std::num::Wrapping;

#[derive(Clone)]
struct QuickRng(Wrapping<u32>);

impl RngCore for QuickRng {
    fn next_u32(&mut self) -> u32 {
        self.0 = self.0 * Wrapping(65539_u32) >> 1;
        self.0.0
    }

    fn next_u64(&mut self) -> u64 {
        impls::next_u64_via_u32(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}

impl SeedableRng for QuickRng {
    type Seed = [u8; 4];
    
    fn from_seed(seed: Self::Seed) -> Self {
        Self(Wrapping(
            ((seed[0] as u32) << 24) |
            ((seed[1] as u32) << 16) |
            ((seed[2] as u32) <<  8) |
            ((seed[3] as u32) <<  0)

        ))
    }
}