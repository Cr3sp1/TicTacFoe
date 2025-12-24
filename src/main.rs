use tic_tac_foe::cli::ask_move;
use tic_tac_foe::game::{Board, Mark};

fn main() {
    let mut board = Board::new();
    println!();
    println!("{}", board);
    println!();

    let mut active_player = Mark::X;
    loop {
        println!("{active_player} is playing");
        let mut player_move: Option<(usize, usize)> = None;
        while player_move == None {
            player_move = ask_move(&board);
        }

        let (row, col) = player_move.unwrap();
        board.set(row, col, Some(active_player));

        active_player = match active_player {
            Mark::X => Mark::O,
            Mark::O => Mark::X,
        };
        println!();
        println!("{}", board);
        println!();

        if let Some(player) = board.check_all() {
            println!("Player {} wins!", player);
            break;
        }

        if board.check_complete() {
            println!("Game ended as a draw");
            break;
        }
    }
}
