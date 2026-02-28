use super::*;
use crate::ai::Move::Base;
use crate::ai::{Game, Move};
use std::fmt;
/// A 3x3 tic-tac-toe board.
///
/// The board is represented as a flat array of 9 cells, where each cell
/// can contain either a mark (X or O) or be empty (None).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmallBoard {
    cells: [Option<Mark>; 9],
    pub state: GameState,
}

impl SmallBoard {
    /// Creates a new empty board with all cells set to None.
    pub fn new() -> Self {
        SmallBoard {
            cells: [None; 9],
            state: GameState::Playing,
        }
    }

    /// Sets the mark at the specified position.
    ///
    /// # Arguments
    /// * `row` - Row index (0-2)
    /// * `col` - Column index (0-2)
    /// * `mark` - The mark to place (Some(Mark::X), Some(Mark::O), or None)
    ///
    /// # Panics
    /// Panics if row or col is greater than 3.
    pub fn set(&mut self, row: usize, col: usize, mark: Option<Mark>) {
        if row > 3 || col > 3 {
            panic!("Tried to access board position ({row}, {col}) which is out of bounds.");
        }
        self.cells[row * 3 + col] = mark;
    }

    /// Makes a move on the board at the specified position.
    ///
    /// Places the given mark at the specified row and column, then checks
    /// if the move resulted in a win or draw and updates the game state.
    ///
    /// # Arguments
    /// * `row` - Row index (0-2)
    /// * `col` - Column index (0-2)
    /// * `mark` - The mark to place (Mark::X or Mark::O)
    ///
    /// # Panics
    /// * Panics if the game is already over (state is not GameState::Playing)
    /// * Panics if the specified position is already occupied
    pub fn make_move(&mut self, row: usize, col: usize, mark: Mark) {
        if self.state != GameState::Playing {
            panic!("Error: tried making a move on a completed board.");
        }
        if self.get(row, col).is_some() {
            panic!("Error: tried making a move on an occupied position.");
        }
        self.set(row, col, Some(mark));
        if check_complete(self) {
            self.state = GameState::Draw;
        }
        if let Some(mark) = check_win(self) {
            self.state = GameState::Won(mark);
        };
    }
}

impl Board for SmallBoard {
    /// Gets the mark at the specified position.
    ///
    /// # Arguments
    /// * `row` - Row index (0-2)
    /// * `col` - Column index (0-2)
    ///
    /// # Panics
    /// Panics if row or col is greater than 3.
    fn get(&self, row: usize, col: usize) -> Option<Mark> {
        if row > 3 || col > 3 {
            panic!("Tried to access board position ({row}, {col}) which is out of bounds.");
        }
        self.cells[row * 3 + col]
    }

    /// Gets whether it is possible to play in the specified position.
    ///
    /// # Arguments
    /// * `row` - Row index (0-2)
    /// * `col` - Column index (0-2)
    ///
    /// # Returns
    /// True if the cell is empty, else False.
    fn is_playable(&self, row: usize, col: usize) -> bool {
        self.get(row, col) == None
    }
}

impl fmt::Display for SmallBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..3 {
            for col in 0..3 {
                let index = row * 3 + col;
                match self.cells[index] {
                    Some(mark) => write!(f, " {} ", mark)?,
                    None => write!(f, " {} ", index)?,
                }
                if col < 2 {
                    write!(f, "|")?;
                }
            }
            if row < 2 {
                writeln!(f)?;
                writeln!(f, "-----------")?;
            }
        }
        Ok(())
    }
}

impl Game for SmallBoard {
    fn play(&mut self, mv: &Move, ai_mark: Mark) {
        let (row, col) = mv.unwrap_base();
        self.make_move(row, col, ai_mark);
    }

    fn get_possible_moves(&self) -> Vec<Move> {
        let mut possible_moves = Vec::new();
        if self.state != GameState::Playing {
            return possible_moves;
        }
        for row in 0..3 {
            for col in 0..3 {
                if self.is_playable(row, col) {
                    possible_moves.push(Base(row, col));
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

#[cfg(test)]
mod tests {
    use super::*;

    impl SmallBoard {
        /// Test helper: Sets an entire row with the provided marks.
        pub fn set_row(&mut self, row: usize, marks: [Option<Mark>; 3]) {
            for col in 0..3 {
                self.set(row, col, marks[col]);
            }
        }

        /// Test helper: Sets an entire column with the provided marks.
        pub fn set_col(&mut self, col: usize, marks: [Option<Mark>; 3]) {
            for row in 0..3 {
                self.set(row, col, marks[row]);
            }
        }
    }

    #[test]
    fn test_check_complete() {
        let mut board = SmallBoard::new();
        assert_eq!(check_complete(&board), false);

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::O)]);
        board.set_row(1, [Some(Mark::X), None, Some(Mark::X)]);
        board.set_row(2, [Some(Mark::O), Some(Mark::O), Some(Mark::X)]);
        assert_eq!(check_complete(&board), false);

        board.set(1, 1, Some(Mark::X));
        assert_eq!(check_complete(&board), true);
    }

    #[test]
    fn test_make_move_win() {
        let mut board = SmallBoard::new();
        assert_eq!(board.state, GameState::Playing);

        // Create a winning row for X
        board.make_move(0, 0, Mark::X);
        board.make_move(1, 0, Mark::O);
        board.make_move(0, 1, Mark::X);
        assert_eq!(board.state, GameState::Playing);
        board.make_move(1, 1, Mark::O);
        board.make_move(0, 2, Mark::X);

        assert_eq!(board.state, GameState::Won(Mark::X));
    }

    #[test]
    fn test_make_move_draw() {
        let mut board = SmallBoard::new();

        // Create a draw scenario
        board.make_move(0, 0, Mark::X);
        board.make_move(0, 1, Mark::O);
        board.make_move(0, 2, Mark::X);
        board.make_move(1, 0, Mark::X);
        board.make_move(1, 1, Mark::O);
        board.make_move(1, 2, Mark::O);
        board.make_move(2, 0, Mark::O);
        board.make_move(2, 1, Mark::X);
        board.make_move(2, 2, Mark::X);

        assert_eq!(board.state, GameState::Draw);
    }

    #[test]
    #[should_panic(expected = "tried making a move on an occupied position")]
    fn test_make_move_occupied_position() {
        let mut board = SmallBoard::new();

        board.make_move(0, 0, Mark::X);
        board.make_move(0, 0, Mark::O); // Should panic
    }

    #[test]
    #[should_panic(expected = "tried making a move on a completed board")]
    fn test_make_move_on_won_board() {
        let mut board = SmallBoard::new();

        // Create a winning scenario
        board.make_move(0, 0, Mark::X);
        board.make_move(1, 0, Mark::O);
        board.make_move(0, 1, Mark::X);
        board.make_move(1, 1, Mark::O);
        board.make_move(0, 2, Mark::X);

        // Try to make a move after game is won
        board.make_move(2, 2, Mark::O); // Should panic
    }

    #[test]
    #[should_panic(expected = "tried making a move on a completed board")]
    fn test_make_move_on_draw_board() {
        let mut board = SmallBoard::new();

        // Create a draw scenario
        board.make_move(0, 0, Mark::X);
        board.make_move(0, 1, Mark::O);
        board.make_move(0, 2, Mark::X);
        board.make_move(1, 0, Mark::X);
        board.make_move(1, 1, Mark::O);
        board.make_move(1, 2, Mark::O);
        board.make_move(2, 0, Mark::O);
        board.make_move(2, 1, Mark::X);
        board.make_move(2, 2, Mark::X);

        // Try to make a move after draw
        board.make_move(0, 0, Mark::X); // Should panic
    }
}
