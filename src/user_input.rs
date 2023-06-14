use crate::board::GridError;
use crate::players::{AiLazy, AiParallel, AiRandom, AiSerial, Human, Player};
use crate::space::{Coord, Piece};
use crate::Board;
use inquire::validator::{ErrorMessage, StringValidator};
use inquire::{
    validator::{CustomTypeValidator, Validation},
    CustomType, InquireError, Text,
};
use inquire::{CustomUserError, Select};
use std::fmt::Display;

pub fn get_board_size() -> usize {
    let size = CustomType::<usize>::new("Enter a size for the board:")
        .with_formatter(&|s| format!("{} by {}", s, s))
        .with_validator(BoardSizeValidator)
        .with_error_message("You must enter a positive whole number")
        .prompt();

    match size {
        Ok(s) => s,
        Err(InquireError::OperationInterrupted) => panic!("User interrupted with ^C"),
        Err(InquireError::OperationCanceled) => panic!("User interrupted with esc"),
        Err(e) => panic!("{}", e),
    }
}

#[derive(Clone)]
struct BoardSizeValidator;
impl CustomTypeValidator<usize> for BoardSizeValidator {
    fn validate(&self, input: &usize) -> Result<Validation, inquire::CustomUserError> {
        match input {
            i if i == &0 => Ok(Validation::Invalid(ErrorMessage::Custom(
                "Must choose a size greater than 0".to_string(),
            ))),
            i if i > &9 => Ok(Validation::Invalid(ErrorMessage::Custom(format!(
                "{} by {} board is too large, choose a number less than 10",
                i, i
            )))),
            _ => Ok(Validation::Valid),
        }
    }
}

enum PlayerSelection {
    Human,
    Random,
    LimitedDepth,
    Deterministic,
    Efficient,
    Comprehensive,
}

impl Display for PlayerSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Human => "Human",
            Self::Random => "Completely random AI",
            Self::LimitedDepth => "Limited-depth AI",
            Self::Deterministic => "Deterministic perfect AI",
            Self::Efficient => "Somewhat predictable perfect AI",
            Self::Comprehensive => "Unpredictable perfect AI",
        };
        write!(f, "{}", str)
    }
}

impl PlayerSelection {
    pub fn to_player(&self, piece: Piece, board_size: usize) -> Box<dyn Player> {
        match self {
            Self::Human => Box::new(Human::new(&get_name(piece), piece)),
            Self::Random => Box::new(AiRandom::new(piece)),
            Self::LimitedDepth => {
                let mut new_player =
                    Box::new(AiSerial::new(board_size, piece, get_depth(board_size)));
                new_player.load_strategy();
                new_player
            }
            Self::Deterministic => {
                let mut new_player = Box::new(AiLazy::new(board_size, piece, true));
                new_player.load_strategy();
                new_player
            }
            Self::Efficient => {
                let mut new_player = Box::new(AiParallel::new(board_size, piece));
                // new_player.load_strategy();
                new_player
            }
            Self::Comprehensive => {
                let mut new_player = Box::new(AiSerial::new(board_size, piece, usize::MAX));
                new_player.load_strategy();
                new_player
            }
        }
    }

    pub fn variants() -> Vec<Self> {
        vec![
            Self::Human,
            Self::Random,
            Self::LimitedDepth,
            Self::Deterministic,
            Self::Efficient,
            Self::Comprehensive,
        ]
    }
}

pub fn get_player(piece: Piece, board_size: usize) -> Box<dyn Player> {
    let message = format!("Select a player type for {}:", piece);
    let player_choice = Select::new(&message, PlayerSelection::variants()).prompt();

    match player_choice {
        Ok(p) => p.to_player(piece, board_size),
        Err(InquireError::OperationInterrupted) => panic!("User interrupted with ^C"),
        Err(InquireError::OperationCanceled) => panic!("User interrupted with esc"),
        Err(e) => panic!("{}", e),
    }
}

fn get_depth(board_size: usize) -> usize {
    let depth = CustomType::<usize>::new("Enter a depth for AI analysis:")
        .with_formatter(&|d| format!("{} moves", d))
        .with_help_message("How many moves ahead the AI should consider")
        .with_validator(DepthValidator { board_size })
        .with_error_message("You must enter a positive whole number")
        .prompt();

    match depth {
        Ok(d) => d,
        Err(InquireError::OperationInterrupted) => panic!("User interrupted with ^C"),
        Err(InquireError::OperationCanceled) => panic!("User interrupted with esc"), // Possibly refactor to allow this to back up one prompt level
        Err(e) => panic!("{}", e),
    }
}

#[derive(Clone)]
struct DepthValidator {
    board_size: usize,
}
impl CustomTypeValidator<usize> for DepthValidator {
    fn validate(&self, input: &usize) -> Result<Validation, inquire::CustomUserError> {
        let max_moves = self.board_size * self.board_size;
        match input {
            i if i == &0 => Ok(Validation::Invalid(ErrorMessage::Custom(
                "Must choose a number greater than 0".to_string(),
            ))),
            i if i > &max_moves => Ok(Validation::Invalid(ErrorMessage::Custom(format!(
                "Board only has {} spaces on it",
                max_moves
            )))),
            _ => Ok(Validation::Valid),
        }
    }
}

fn get_name(piece: Piece) -> String {
    let name = Text::new("Enter a name for the player:")
        .with_default(&format!("Player {}", piece))
        .with_formatter(&|n| format!("{}", piece.colorize(n)))
        .prompt();

    match name {
        Ok(n) => n,
        Err(InquireError::OperationInterrupted) => panic!("User interrupted with ^C"),
        Err(InquireError::OperationCanceled) => panic!("User interrupted with esc"), // Possibly refactor to allow this to back up one prompt level
        Err(e) => panic!("{}", e),
    }
}

#[derive(Clone)]
struct MoveSyntaxValidator;
impl StringValidator for MoveSyntaxValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        let standard_err = Ok(Validation::Invalid(ErrorMessage::Custom(
            "Invalid space. Enter in the form \"A1\"".to_string(),
        )));
        let mut chars = input.chars();

        // first character
        match chars.next() {
            None => {
                return standard_err;
            }
            Some(c) => {
                if !c.is_ascii_alphabetic() {
                    return standard_err;
                }
            }
        }

        // second character
        match chars.next() {
            None => {
                return standard_err;
            }
            Some(c) => {
                if !c.is_numeric() {
                    return standard_err;
                }
            }
        }

        if chars.next().is_some() {
            return standard_err;
        }

        Ok(Validation::Valid)
    }
}

#[derive(Clone)]
struct MovePlacementValidator {
    b: Board,
}

impl StringValidator for MovePlacementValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        let input = input.to_uppercase();

        let mut chars = input.chars();

        // first character
        let col_char = chars.next().expect("Input was not validated properly");
        let col = col_char as u8 - b'A';

        // second character
        let row_char = chars.next().expect("Input was not validated properly");
        let row = row_char as u8 - b'1';

        match self.b.clone().place(
            Piece::X,
            Coord {
                row: row.into(),
                col: col.into(),
            },
        ) {
            Err(GridError::RowIndexOutOfBounds { .. }) => Ok(Validation::Invalid(
                ErrorMessage::Custom(format!("Invalid space, row {} out of bounds", row_char)),
            )),
            Err(GridError::ColIndexOutOfBounds { .. }) => Ok(Validation::Invalid(
                ErrorMessage::Custom(format!("Invalid space, column {} out of bounds", col_char)),
            )),
            Err(GridError::SpaceOccupied { .. }) => Ok(Validation::Invalid(ErrorMessage::Custom(
                format!("Space {}{} is already occupied", col_char, row_char),
            ))),
            Ok(_) => Ok(Validation::Valid),
        }
    }
}

impl MovePlacementValidator {
    // assumes string is already in the correct format
    fn coord_from_str(move_str: &str) -> Coord {
        let move_str = move_str.to_uppercase();

        let row: u8;
        let col: u8;

        let mut chars = move_str.chars();
        if let Some(col_char) = chars.next() {
            col = col_char as u8 - b'A';
        } else {
            panic!("Input was not validated properly")
        }

        // second character
        if let Some(row_char) = chars.next() {
            row = row_char as u8 - b'1';
        } else {
            panic!("Input was not validated properly")
        }

        Coord {
            row: row.into(),
            col: col.into(),
        }
    }
}

pub fn get_move(name: &str, b: &Board) -> Coord {
    let validators: Vec<Box<dyn StringValidator>> = vec![
        Box::new(MoveSyntaxValidator),
        Box::new(MovePlacementValidator { b: b.clone() }),
    ];
    let move_result = Text::new(&format!("{}, enter your move: ", name))
        .with_validators(&validators)
        .prompt();

    match move_result {
        Ok(move_str) => MovePlacementValidator::coord_from_str(&move_str),
        _ => panic!("Encountered an error"),
    }
}
