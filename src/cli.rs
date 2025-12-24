use crate::game::Board;
use std::io::{self, Write};

fn get_player_input() -> Option<usize> {
    print!("Enter a position (0-8): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;

    input.trim().parse().ok()
}

pub fn ask_move(board: &Board) -> Option<(usize, usize)> {
    let move_pos;

    match get_player_input() {
        Some(pos) if pos <= 8 => {
            // Valid input
            println!("You chose position {}", pos);
            move_pos = pos;
        }
        _ => {
            println!("Invalid input! Please enter a number between 0 and 8!");
            return None;
        }
    }

    let move_row = move_pos / 3;
    let move_col = move_pos % 3;
    if board.get(move_row, move_col).is_some() {
        println!("Invalid input! Position {move_pos} is already occupied!");
        return None;
    }

    Some((move_row, move_col))
}
