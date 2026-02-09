pub mod mcts;
pub mod random;
pub mod simple;

use crate::ai::Move::{Base, Ultimate};
use crate::game::base::SmallBoard;
use crate::game::{Board, GameState, Mark};
use crate::utils::Position;

#[derive(Clone, Copy, Debug)]
pub enum Move {
    Base(Position),
    Ultimate(Position, Position),
}

impl Move {
    pub fn unpack_base(&self) -> (usize, usize) {
        match &self {
            Base(small) => (small.row, small.col),
            _ => panic!("Expected Base move, got Ultimate."),
        }
    }

    pub fn unpack_ultimate(&self) -> (usize, usize, usize, usize) {
        match &self {
            Ultimate(small, big) => (small.row, small.col, big.row, big.col),
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

impl Game for SmallBoard {
    fn play(&mut self, mv: &Move, ai_mark: Mark) {
        let (row, col) = mv.unpack_base();
        self.make_move(row, col, ai_mark);
    }

    fn get_possible_moves(&self) -> Vec<Move> {
        let mut possible_moves = Vec::new();
        for row in 0..3 {
            for col in 0..3 {
                if self.is_playable(row, col) {
                    possible_moves.push(Base(Position { row, col }));
                }
            }
        }
        possible_moves
    }

    fn score(&self, mark: Mark) -> i8 {
        if let GameState::Won(winning_mark) = self.state {
            return if mark == winning_mark { 1 } else { -1 };
        }
        0
    }

    fn get_state(&self) -> GameState {
        self.state
    }
}
