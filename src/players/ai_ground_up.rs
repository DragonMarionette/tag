use ciborium::{de, ser};
use itertools::Itertools;
use serde::{Serialize, Deserialize};
use rand::seq::SliceRandom;
use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::fs::File;
use std::num;

use super::{available_spaces, MoveValue, Player};
use crate::space::{Coord, Piece};
use crate::Board;
use crate::ScrambledBoard;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveAnalysis {
    pub evaluation: MoveValue,
    pub move_options: Vec<Coord>
}

pub struct AiGroundUp {
    size: usize,
    piece: Piece,
    known_boards: HashMap<Board, MoveAnalysis>,
}

impl Display for AiGroundUp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.piece.colorize("AI"), self.piece)
    }
}

impl Player for AiGroundUp {
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

impl AiGroundUp {
    pub fn new(size: usize, piece: Piece) -> Self {
        Self {
            size,
            piece,
            known_boards: HashMap::new(),
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

        let analysis = self.analyze(&key);

        let chosen_move = analysis
            .move_options
            .choose(&mut rand::thread_rng())
            .unwrap();
        scrambled.space_at(chosen_move.clone()).unwrap().to_coord()
    }

    fn analyze(&mut self, key: &Board) -> MoveAnalysis {
        if let Some(analysis) = self.known_boards.get(key) {
            analysis.clone()
        } else {
            self.build_strategy();
            self.analyze(key)
        }
    }

    fn build_strategy(&mut self) {
        let starting_piece = self.piece;
        let mut current_piece = match self.size % 2 {
            0 => self.piece.inverse(),
            1 => self.piece,
            _ => panic!("self.size % 2 was neither 0 nor 1"),
        };

        // for full boards
        for b in BoardIterator::new(self.size, starting_piece)
            .filter(|b| ScrambledBoard::from(b.clone()).is_standard())
        {
            let evaluation = if b.has_win(current_piece) {
                if current_piece == self.piece {
                    MoveValue::Win(0)
                } else {
                    MoveValue::Lose(0)
                }
            } else {
                MoveValue::Tie(0)
            };

            let analysis = MoveAnalysis {
                evaluation,
                move_options: Vec::new(),
            };
            self.known_boards.insert(b, analysis);
        }

        // for unfull boards
        for num_empty in 1..=(self.size*self.size) {
            println!("num_empty = {}", num_empty);
            current_piece = current_piece.inverse();
            for (b, scrambled) in BoardIterator::from_move_number(self.size, starting_piece, num_empty)
                .map(|b| (b.clone(), ScrambledBoard::from(b)))
                .filter(|(_, sb)| sb.is_standard())
            {
                let evaluation: MoveValue;
                let move_options: Vec<Coord>;

                if b.has_win(current_piece) {
                    if current_piece == self.piece {
                        evaluation = MoveValue::Win(0);
                        move_options = Vec::new();
                    } else {
                        evaluation = MoveValue::Lose(0);
                        move_options = Vec::new();
                    }
                } else {
                    let mut new_analyses: Vec<(Coord, MoveValue)> = Vec::new();

                    for move_option in available_spaces(&b) {
                        let mut sb = scrambled.clone();
                        sb.place(current_piece.inverse(), move_option).expect("available_spaces yielded an illegal Coord");
                        sb.fully_standardize();

                        let k = &Board::from(sb);
                        let mut lower_evaluation = self.known_boards.get(k)
                            .expect(&format!("Requested this unknown board from known_boards:\n{}", k))
                            .evaluation.clone();
                        lower_evaluation = match lower_evaluation {
                            MoveValue::Lose(v) => MoveValue::Lose(v + 1),
                            MoveValue::Tie(v) => MoveValue::Tie(v + 1),
                            MoveValue::Unknown(v) => MoveValue::Unknown(v + 1),
                            MoveValue::Win(v) => MoveValue::Win(v + 1),
                        };
                        new_analyses.push((move_option, lower_evaluation));
                    }

                    if current_piece == self.piece {
                        evaluation = new_analyses.iter()
                            .map(|(_c, v)| v.clone())
                            .max()
                            .expect(&format!("new_analyses was empty for this board:\n{}", b));
                        new_analyses = new_analyses.into_iter()
                            .filter(|(_c, v)| v == &evaluation)
                            .collect();
                    } else {
                        evaluation = new_analyses.iter()
                            .map(|(_c, v)| v.clone())
                            .min()
                            .expect(&format!("new_analyses was empty for this board:\n{}", b));
                        new_analyses = new_analyses.into_iter()
                            .filter(|(_c, v)| v == &evaluation)
                            .collect();
                    }

                    move_options = new_analyses.into_iter().map(|(c, _v)| c).collect();


                }

                let new_analysis = MoveAnalysis {
                    evaluation,
                    move_options,
                };
                
                self.known_boards.insert(b, new_analysis);
            } // end for boards
        }// end for num_empty
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
            "strategies/ground-up-s{}-p{}.cbor",
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


#[derive(Clone)]
pub struct BoardIterator {
    current_vec: VecDeque<Piece>,
    len: usize,
    boards_served: usize,
}

impl Iterator for BoardIterator {
    type Item = Board;
    fn next(&mut self) -> Option<Self::Item> {
        if self.boards_served >= self.len {
           return None;
        }

        let b = Board::from(self.current_vec.clone());
        self.boards_served += 1;

        self.permute();

        Some(b)
    }
}

impl BoardIterator {
    pub fn new(board_size: usize, starting_piece: Piece) -> Self {
        Self::from_move_number(board_size, starting_piece, 0)
    }

    pub fn from_move_number(board_size: usize, starting_piece: Piece, num_empty: usize) -> Self {
        let num_filled = board_size*board_size - num_empty;
        match starting_piece {
            Piece::X => Self::from_counts((num_filled + 1)/2, num_filled/2, num_empty),
            Piece::O => Self::from_counts(num_filled/2, (num_filled + 1)/2, num_empty),
            Piece::Empty => panic!("starting_piece was Piece::Empty")
        }
    }

    pub fn from_counts(count_x: usize, count_o: usize, count_empty: usize) -> Self {
        let mut current_vec = vec![Piece::X; count_x];
        current_vec.append(&mut vec![Piece::O; count_o]);
        current_vec.append(&mut vec![Piece::Empty; count_empty]);
        
        Self {
            current_vec: current_vec.into(),
            len: num_integer::multinomial(&[count_x, count_o, count_empty]),
            boards_served: 0
        }

    }

    // implmentation of Aaron Williams's algorithm for permutations of a multiset
    // detailed in https://dl.acm.org/doi/10.5555/1496770.1496877
    // summarized at https://github.com/ekg/multipermute
    fn permute(&mut self){
        match self.current_vec.len() {
            0 => (),
            _ => {
                let a = self.current_vec.pop_front().unwrap();
                for (idx_b, (b, c)) in self.current_vec.iter().tuple_windows().enumerate() {
                    
                    if b < c {
                        if &a > b {
                            self.current_vec.insert(idx_b + 1, a);
                        } else {
                            let idx_c = idx_b + 1;
                            self.current_vec.insert(idx_c + 1, a);
                        }
                        return;
                    }
                }
                self.current_vec.push_back(a);
            }
        }
    }
}
