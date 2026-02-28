pub mod mcts;
pub mod random;
pub mod simple;

use crate::ai::Move::{Base, Ultimate};
use crate::ai::mcts::MCTSAi;
use crate::ai::random::random_move;
use crate::ai::simple::SimpleAi;
use crate::game::base::SmallBoard;
use crate::game::ultimate::BigBoard;
use crate::game::{GameState, Mark};

#[derive(Clone, Debug, PartialEq)]
pub enum AI {
    Weak(Mark),
    Medium(SimpleAi),
    StrongTTT(MCTSAi<SmallBoard>),
    StrongUTT(MCTSAi<BigBoard>),
}

impl AI {
    pub fn choose_move_ttt(&mut self, board: &SmallBoard) -> Move {
        match self {
            AI::Weak(_) => random_move(board),
            AI::Medium(ai) => ai.choose_move(board),
            AI::StrongTTT(ai) => ai.choose_move(board),
            _ => panic!("Invalid AI."),
        }
    }

    pub fn choose_move_utt(&mut self, board: &BigBoard) -> Move {
        match self {
            AI::Weak(_) => random_move(board),
            AI::Medium(ai) => ai.choose_move(board),
            AI::StrongUTT(ai) => ai.choose_move(board),
            _ => panic!("Invalid AI."),
        }
    }

    pub fn get_mark(&self) -> Mark {
        match self {
            AI::Weak(mark) => *mark,
            AI::Medium(ai) => ai.ai_mark,
            AI::StrongTTT(ai) => ai.ai_mark,
            AI::StrongUTT(ai) => ai.ai_mark,
        }
    }

    pub fn switch_starting_mark(&mut self) {
        match self {
            AI::StrongTTT(ai) => ai.switch_starting_mark(),
            AI::StrongUTT(ai) => ai.switch_starting_mark(),
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        match self {
            AI::StrongTTT(ai) => ai.reset(),
            AI::StrongUTT(ai) => ai.reset(),
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    Base(usize, usize),
    Ultimate(usize, usize, usize, usize),
}

impl Move {
    pub fn unwrap_base(&self) -> (usize, usize) {
        match &self {
            Base(row, col) => (*row, *col),
            _ => panic!("Expected Base move, got Ultimate."),
        }
    }

    pub fn unwrap_ultimate(&self) -> (usize, usize, usize, usize) {
        match &self {
            Ultimate(board_row, board_col, cell_row, cell_col) => {
                (*board_row, *board_col, *cell_row, *cell_col)
            }
            _ => panic!("Expected Ultimate move, got Base."),
        }
    }
}

pub trait Game {
    fn play(&mut self, mv: &Move, ai_mark: Mark);
    fn get_possible_moves(&self) -> Vec<Move>;
    fn score(&self, mark: Mark) -> i8;
    fn get_state(&self) -> GameState;
}
