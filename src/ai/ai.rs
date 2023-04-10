use std::{collections::HashMap, ops::Deref};
use rand::seq::SliceRandom;

use crate::{board::Board, piece::Piece};

use super::scrambled_board::{ScrambledBoard, Space, Coord};


// enum MoveValue {
//     Win(u8), // number of moves
//     Lose(u8),
//     Tie(u8),
//     Unknown(u8)
// }

#[derive(Debug, Clone)]
struct MoveAnalysis {
    evaluation: i8, // should range from -100 to 100
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

        self.analyze(&key);

        // println!("self.known_boards = {:?}", self.known_boards);
        let move_options = self.known_boards.get(&key).unwrap().move_options.clone();
        let chosen_move = *move_options.choose(&mut self.rng).unwrap();
        Coord::from_space(&scrambled.space_at(chosen_move).unwrap())
    }

    fn analyze(&mut self, b: &Board) -> MoveAnalysis{ // analyze the given move, saving the resulting analysis to self.known_boards
        self.analyze_recursive(b, self.depth)
    }

    fn analyze_recursive(&mut self, b: &Board, depth_remaining: usize) -> MoveAnalysis { // assumes it is getting an already-standardized board
        

        if let Some(analysis) = self.known_boards.get(b) { // b already computed to sufficient depth
            if analysis.depth_used >= depth_remaining {
                return analysis.clone();
            }
        }

        if b.has_win(self.piece.inverse()) { // b already has other player winning
            let new_analysis = MoveAnalysis {
                evaluation: -100,
                move_options: vec![],
                depth_used: self.depth, // max depth because no need to ever reanalyze this position deeper
            };
            self.known_boards.insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        if b.is_full() {
            let new_analysis = MoveAnalysis {
                evaluation: 0,
                move_options: vec![],
                depth_used: self.depth, // max depth because no need to ever reanalyze this position deeper
            };
            self.known_boards.insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        if depth_remaining == 0 {
            let new_analysis = MoveAnalysis {
                evaluation: self.evaluate_stupidly(b),
                move_options: available_spaces(b),
                depth_used: 0,
            };
            self.known_boards.insert(b.clone(), new_analysis.clone());
            return new_analysis;
        }

        // recursive case
        // not really pruned against looking deeper when a win is already found, maybe do that
        let mut new_analyses: Vec<(Coord, MoveAnalysis)> = Vec::new();
        for c in available_spaces(b) {
            let (row, col) = (c.row, c.col);
            let mut b = b.clone();
            b.place(self.piece, row, col).unwrap();
            b.invert();
            let mut scrambled = ScrambledBoard::from_board(&b);
            scrambled.standardize();
            let mut lower_analysis = self.analyze_recursive(&scrambled.to_board(), depth_remaining-1);

            lower_analysis.evaluation *= -1;

            new_analyses.push((c, lower_analysis))
        }

        let shallowest_depth = new_analyses.iter()
            .map(|a| a.1.depth_used)
            .min().unwrap();
        let depth_used = shallowest_depth + 1;

        // println!("{}", b);

        let best_evaluation = new_analyses.iter().map(|a| a.1.evaluation)
            .max().unwrap();
        // filter to keep only the best-evaluated moves
        new_analyses = new_analyses.iter()
            .filter_map(|a| if a.1.evaluation == best_evaluation {Some(a.clone())} else {None})
            .collect();

        let move_options = new_analyses.iter()
            // .filter(|a| a.1.depth_used == shallowest_depth)
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

    fn evaluate_stupidly(&self, _b: &Board) -> i8 { // Optional TODO: use fraction of empty spaces left
        0
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
