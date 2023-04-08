use std::io;
use crate::{piece::Piece, board::Board};

pub fn play_game() {
    /* 
    get size for board
    loop between players
        ask for move
        while invalid text: get move
        try and make move
            if fail, ask again
        break on win
    
    */
    
    let mut board_size_string = String::new();
    print!("Enter a size for the board: ");
    while let Err(_) = io::stdin().read_line(&mut board_size_string) {
        println!("Unable to read your input. Try again.");
        print!("Enter a size for the board: ");
        board_size_string.clear();
    }
    println!("got input of {}", board_size_string);
}