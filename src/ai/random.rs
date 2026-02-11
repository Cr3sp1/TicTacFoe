pub struct RandomAI {}
use crate::ai::{Game, Move};
use rand::prelude::*;

/// Chooses a random available for the AI on the given board.
///
/// # Arguments
/// * `board` - The current game board state
///
/// # Returns
/// A tuple (row, col) representing the chosen move
///
/// # Panics
/// Panics if there are no available moves on the board
pub fn random_move<T>(board: T) -> Move
where
    T: Game,
{
    // find all available moves
    let ai_moves = board.get_possible_moves();
    if ai_moves.is_empty() {
        panic!("No available moves found by RandomAi");
    }

    let mut rng = rand::rng();
    *ai_moves.choose(&mut rng).unwrap()
}
