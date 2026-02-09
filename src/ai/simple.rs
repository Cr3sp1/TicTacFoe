use crate::ai::{Game, Move};
use crate::game::Mark;
use rand::prelude::*;

/// A simple AI opponent for Tic-Tac-Toe that uses basic strategy.
///
/// The AI prioritizes moves in the following order:
/// 1. Win if possible
/// 2. Block opponent's winning move
/// 3. Choose randomly from available positions
pub struct SimpleAi {
    pub ai_mark: Mark,
    enemy_mark: Mark,
}

impl SimpleAi {
    /// Creates a new SimpleAi with the given mark.
    ///
    /// # Arguments
    /// * `ai_mark` - The mark (X or O) that the AI will play as
    ///
    /// # Example
    /// ```
    /// use tic_tac_foe::{ai::simple::SimpleAi, game::Mark};
    /// let ai = SimpleAi::new(Mark::X);
    /// assert_eq!(ai.ai_mark, Mark::X);
    /// ```
    pub fn new(ai_mark: Mark) -> SimpleAi {
        SimpleAi {
            ai_mark,
            enemy_mark: match ai_mark {
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
    pub fn choose_move<T>(&self, board: T) -> Move
    where
        T: Game + Clone,
    {
        // find all available moves
        let ai_moves = board.get_possible_moves();
        if ai_moves.is_empty() {
            panic!("No available moves found by SimpleAi");
        }
        let mut non_losing_moves = ai_moves.clone();

        // save original board score
        let original_ai_score = board.score(self.ai_mark);

        // check result of all for available moves starting from the last in moves vectors
        'outer: for (i, mv) in ai_moves.iter().enumerate().rev() {
            let mut board_i = board.clone();
            board_i.play(mv, self.ai_mark);

            // if a move improves score (wins a board) play it
            if board_i.score(self.ai_mark) > original_ai_score {
                return *mv;
            }
            // else check that the move doesn't let the enemy win
            let enemy_moves = board_i.get_possible_moves();
            for enemy_mv in enemy_moves.iter() {
                let mut board_j = board_i.clone();
                board_j.play(enemy_mv, self.enemy_mark);
                // if score is worse than the starting one the move is a losing one
                if board_j.score(self.ai_mark) < original_ai_score {
                    non_losing_moves.remove(i);
                    continue 'outer;
                }
            }
        }

        let mut rng = rand::rng();

        // if there are non-losing moves return one of them
        if let Some(mv) = non_losing_moves.choose(&mut rng) {
            return *mv;
        }
        // else move at random
        *ai_moves.choose(&mut rng).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::base::SmallBoard;

    #[test]
    fn test_ai_takes_winning_move() {
        let mut board = SmallBoard::new();
        // Set up board where AI (O) can win
        board.set(0, 0, Some(Mark::O));
        board.set(0, 1, Some(Mark::O));
        // Position (0, 2) would be winning move

        let ai = SimpleAi::new(Mark::O);
        let (row, col) = ai.choose_move(board.clone()).unpack_base();

        assert_eq!((row, col), (0, 2));
    }

    #[test]
    fn test_ai_blocks_opponent_win() {
        let mut board = SmallBoard::new();
        // Set up board where player (X) is about to win
        board.set(0, 0, Some(Mark::X));
        board.set(0, 1, Some(Mark::X));
        // AI must block at (0, 2)

        let ai = SimpleAi::new(Mark::O);
        let (row, col) = ai.choose_move(board.clone()).unpack_base();

        assert_eq!((row, col), (0, 2));
    }

    #[test]
    #[should_panic(expected = "No available moves found by SimpleAi")]
    fn test_ai_panics_on_full_board() {
        let mut board = SmallBoard::new();
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
