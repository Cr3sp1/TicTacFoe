pub mod mcts;
pub mod random;
pub mod simple;

use crate::ai::Move::{Base, Ultimate};
use crate::ai::random::random_move;
use crate::ai::simple::SimpleAi;
use crate::game::{GameState, Mark};

pub enum AI {
    Weak(Mark),
    Medium(SimpleAi),
}

impl AI {
    pub fn choose_move<T>(&self, board: T) -> Move
    where
        T: Game + Clone,
    {
        match &self {
            AI::Weak(_) => random_move(board),
            AI::Medium(ai) => ai.choose_move(board),
        }
    }

    pub fn get_mark(&self) -> Mark {
        match &self {
            AI::Weak(mark) => *mark,
            AI::Medium(ai) => ai.ai_mark,
        }
    }
}

#[derive(Clone, Copy, Debug)]
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
