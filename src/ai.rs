use crate::game::{Board, Mark};
use rand::prelude::*;
use std::vec::Vec;

pub struct SimpleAi {
    pub ai_mark: Mark,
    player_mark: Mark,
}

impl SimpleAi {
    pub fn new(ai_mark: Mark) -> SimpleAi {
        SimpleAi {
            ai_mark,
            player_mark: match ai_mark {
                Mark::O => Mark::X,
                Mark::X => Mark::O,
            },
        }
    }

    pub fn choose_move(&self, mut board: Board) -> (usize, usize) {
        // find all available moves
        let available = available_moves(&board);
        if available.is_empty() {
            panic!("No available moves found by SimpleAi");
        }

        // check for available wins
        for &(row, col) in available.iter() {
            board.set(row, col, Some(self.ai_mark));
            match board.check_all() {
                Some(_) => {
                    return (row, col);
                }
                _ => {}
            };
            board.set(row, col, None);
        }

        // check for possible losses
        for &(row, col) in available.iter() {
            board.set(row, col, Some(self.player_mark));
            match board.check_all() {
                Some(_) => {
                    return (row, col);
                }
                _ => {}
            };
            board.set(row, col, None);
        }

        // move at random
        let mut rng = rand::rng();
        *available.choose(&mut rng).unwrap()
    }
}

fn available_moves(board: &Board) -> Vec<(usize, usize)> {
    let mut moves: Vec<(usize, usize)> = Vec::new();
    for row in 0..3 {
        for col in 0..3 {
            if board.get(row, col).is_none() {
                moves.push((row, col));
            }
        }
    }
    moves
}
