use crate::game::Mark;
use crate::game::base::Board;
use rand::prelude::*;
use std::vec::Vec;

/// A simple AI opponent for Tic-Tac-Toe that uses basic strategy.
///
/// The AI prioritizes moves in the following order:
/// 1. Win if possible
/// 2. Block opponent's winning move
/// 3. Choose randomly from available positions
pub struct SimpleAi {
    pub ai_mark: Mark,
    player_mark: Mark,
}

impl SimpleAi {
    /// Creates a new SimpleAi with the given mark.
    ///
    /// # Arguments
    /// * `ai_mark` - The mark (X or O) that the AI will play as
    ///
    /// # Example
    /// ```
    /// use tic_tac_foe::{ai::SimpleAi, game::Mark};
    /// let ai = SimpleAi::new(Mark::X);
    /// assert_eq!(ai.ai_mark, Mark::X);
    /// ```
    pub fn new(ai_mark: Mark) -> SimpleAi {
        SimpleAi {
            ai_mark,
            player_mark: match ai_mark {
                Mark::O => Mark::X,
                Mark::X => Mark::O,
            },
        }
    }

    /// Chooses the best move for the AI on the given board.
    ///
    /// # Arguments
    /// * `board` - The current game board state
    ///
    /// # Returns
    /// A tuple (row, col) representing the chosen move
    ///
    /// # Panics
    /// Panics if there are no available moves on the board
    pub fn choose_move(&self, mut board: Board) -> (usize, usize) {
        // find all available moves
        let available = available_moves(&board);
        if available.is_empty() {
            panic!("No available moves found by SimpleAi");
        }

        // check for available wins
        for &(row, col) in available.iter() {
            board.set(row, col, Some(self.ai_mark));
            match board.check_win() {
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
            match board.check_win() {
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

/// Returns a list of all empty positions on the board.
///
/// # Arguments
/// * `board` - The board to check for available moves
///
/// # Returns
/// A vector of (row, col) tuples representing empty positions
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_takes_winning_move() {
        let mut board = Board::new();
        // Set up board where AI (O) can win
        board.set(0, 0, Some(Mark::O));
        board.set(0, 1, Some(Mark::O));
        // Position (0, 2) would be winning move

        let ai = SimpleAi::new(Mark::O);
        let (row, col) = ai.choose_move(board.clone());

        assert_eq!((row, col), (0, 2));
    }

    #[test]
    fn test_ai_blocks_opponent_win() {
        let mut board = Board::new();
        // Set up board where player (X) is about to win
        board.set(0, 0, Some(Mark::X));
        board.set(0, 1, Some(Mark::X));
        // AI must block at (0, 2)

        let ai = SimpleAi::new(Mark::O);
        let (row, col) = ai.choose_move(board.clone());

        assert_eq!((row, col), (0, 2));
    }

    #[test]
    #[should_panic(expected = "No available moves found by SimpleAi")]
    fn test_ai_panics_on_full_board() {
        let mut board = Board::new();
        // Fill the entire board
        for row in 0..3 {
            for col in 0..3 {
                board.set(row, col, Some(Mark::X));
            }
        }

        let ai = SimpleAi::new(Mark::O);
        ai.choose_move(board);
    }
}
