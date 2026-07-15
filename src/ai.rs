/// Monte Carlo tree-search AI implementation.
pub mod mcts;
/// Random move selection.
pub mod random;
/// Rule-based classic tic-tac-toe AI.
pub mod simple;

use crate::ai::Move::{Base, Ultimate};
use crate::ai::mcts::MCTSAi;
use crate::ai::random::random_move;
use crate::ai::simple::SimpleAi;
use crate::game::base::SmallBoard;
use crate::game::ultimate::BigBoard;
use crate::game::{GameState, Mark};

/// Available AI strategies for classic and Ultimate tic-tac-toe.
#[derive(Clone, Debug, PartialEq)]
pub enum AI {
    /// Chooses uniformly from legal moves.
    Weak(Mark),
    /// Uses immediate wins and blocks in classic tic-tac-toe.
    Medium(SimpleAi),
    /// Uses Monte Carlo tree search on a classic board.
    StrongTTT(MCTSAi<SmallBoard>),
    /// Uses Monte Carlo tree search on an Ultimate board.
    StrongUTT(MCTSAi<BigBoard>),
}

impl AI {
    /// Chooses a move for a classic tic-tac-toe board.
    pub fn choose_move_ttt(&mut self, board: &SmallBoard) -> Move {
        match self {
            AI::Weak(_) => random_move(board),
            AI::Medium(ai) => ai.choose_move(board),
            AI::StrongTTT(ai) => ai.choose_move(board),
            _ => panic!("Invalid AI."),
        }
    }

    /// Chooses a move for an Ultimate tic-tac-toe board.
    pub fn choose_move_utt(&mut self, board: &BigBoard) -> Move {
        match self {
            AI::Weak(_) => random_move(board),
            AI::Medium(ai) => ai.choose_move(board),
            AI::StrongUTT(ai) => ai.choose_move(board),
            _ => panic!("Invalid AI."),
        }
    }

    /// Returns the mark controlled by this AI.
    pub fn get_mark(&self) -> Mark {
        match self {
            AI::Weak(mark) => *mark,
            AI::Medium(ai) => ai.ai_mark,
            AI::StrongTTT(ai) => ai.ai_mark,
            AI::StrongUTT(ai) => ai.ai_mark,
        }
    }

    /// Switches the starting mark for stateful AI implementations.
    pub fn switch_starting_mark(&mut self) {
        match self {
            AI::StrongTTT(ai) => ai.switch_starting_mark(),
            AI::StrongUTT(ai) => ai.switch_starting_mark(),
            _ => {}
        }
    }

    /// Resets any state retained by the AI.
    pub fn reset(&mut self) {
        match self {
            AI::StrongTTT(ai) => ai.reset(),
            AI::StrongUTT(ai) => ai.reset(),
            _ => {}
        }
    }
}

/// A legal move in either supported game variant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    /// A classic move containing its row and column.
    Base(usize, usize),
    /// An Ultimate move containing board and cell coordinates.
    Ultimate(usize, usize, usize, usize),
}

impl Move {
    /// Returns classic move coordinates.
    ///
    /// # Panics
    /// Panics if this is an Ultimate move.
    pub fn unwrap_base(&self) -> (usize, usize) {
        match &self {
            Base(row, col) => (*row, *col),
            _ => panic!("Expected Base move, got Ultimate."),
        }
    }

    /// Returns Ultimate board and cell coordinates.
    ///
    /// # Panics
    /// Panics if this is a classic move.
    pub fn unwrap_ultimate(&self) -> (usize, usize, usize, usize) {
        match &self {
            Ultimate(board_row, board_col, cell_row, cell_col) => {
                (*board_row, *board_col, *cell_row, *cell_col)
            }
            _ => panic!("Expected Ultimate move, got Base."),
        }
    }
}

/// Common board operations required by AI implementations.
pub trait Game {
    /// Applies a move using the supplied mark.
    fn play(&mut self, mv: &Move, ai_mark: Mark);
    /// Returns every legal move in the current state.
    fn get_possible_moves(&self) -> Vec<Move>;
    /// Evaluates the board from the supplied mark's perspective.
    fn score(&self, mark: Mark) -> i8;
    /// Returns the current game state.
    fn get_state(&self) -> GameState;
}
