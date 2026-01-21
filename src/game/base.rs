use super::*;
use std::fmt;

/// A 3x3 tic-tac-toe board.
///
/// The board is represented as a flat array of 9 cells, where each cell
/// can contain either a mark (X or O) or be empty (None).
#[derive(Copy, Clone)]
pub struct Board {
    cells: [Option<Mark>; 9],
    pub state: GameState,
}

impl Board {
    /// Creates a new empty board with all cells set to None.
    pub fn new() -> Self {
        Board {
            cells: [None; 9],
            state: GameState::Playing,
        }
    }

    /// Gets the mark at the specified position.
    ///
    /// # Arguments
    /// * `row` - Row index (0-2)
    /// * `col` - Column index (0-2)
    ///
    /// # Panics
    /// Panics if row or col is greater than 3.
    pub fn get(&self, row: usize, col: usize) -> Option<Mark> {
        if row > 3 || col > 3 {
            panic!("Tried to access board position ({row}, {col}) which is out of bounds.");
        }
        self.cells[row * 3 + col]
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

    /// Checks if the specified row has three matching marks.
    ///
    /// Returns the winning mark if all three cells in the row match,
    /// or None if they don't match or any cell is empty.
    fn check_row(&self, row: usize) -> Option<Mark> {
        let mark_0 = self.get(row, 0)?;
        for i in 1..3 {
            let mark_i = self.get(row, i)?;
            if mark_i != mark_0 {
                return None;
            }
        }
        Some(mark_0)
    }

    /// Checks if the specified column has three matching marks.
    ///
    /// Returns the winning mark if all three cells in the column match,
    /// or None if they don't match or any cell is empty.
    fn check_col(&self, col: usize) -> Option<Mark> {
        let mark_0 = self.get(0, col)?;
        for i in 1..3 {
            let mark_i = self.get(i, col)?;
            if mark_i != mark_0 {
                return None;
            }
        }
        Some(mark_0)
    }

    /// Checks the top-left to bottom-right diagonal for three matching marks.
    ///
    /// Returns the winning mark if all three cells match, or None otherwise.
    fn check_diag_dexter(&self) -> Option<Mark> {
        let mark_0 = self.get(0, 0)?;
        for i in 1..3 {
            let mark_i = self.get(i, i)?;
            if mark_i != mark_0 {
                return None;
            }
        }
        Some(mark_0)
    }

    /// Checks the top-right to bottom-left diagonal for three matching marks.
    ///
    /// Returns the winning mark if all three cells match, or None otherwise.
    fn check_diag_sinister(&self) -> Option<Mark> {
        let mark_0 = self.get(0, 2)?;
        for i in 1..3 {
            let mark_i = self.get(i, 2 - i)?;
            if mark_i != mark_0 {
                return None;
            }
        }
        Some(mark_0)
    }

    /// Checks all possible winning conditions (rows, columns, and diagonals).
    ///
    /// Returns the winning mark if any winning condition is met, or None if
    /// there is no winner yet.
    fn check_win(&mut self) -> Option<Mark> {
        if let Some(mark) = self.check_diag_dexter() {
            self.state = GameState::Won(mark);
            return Some(mark);
        }
        if let Some(mark) = self.check_diag_sinister() {
            self.state = GameState::Won(mark);
            return Some(mark);
        }
        for i in 0..3 {
            if let Some(mark) = self.check_row(i) {
                self.state = GameState::Won(mark);
                return Some(mark);
            }
            if let Some(mark) = self.check_col(i) {
                self.state = GameState::Won(mark);
                return Some(mark);
            }
        }

        None
    }

    /// Checks if all cells on the board are filled.
    ///
    /// Returns true if every cell contains a mark, false otherwise.
    fn check_complete(&mut self) -> bool {
        for i in 0..9 {
            if self.cells[i].is_none() {
                return false;
            }
        }
        self.state = GameState::Draw;
        true
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
            panic!("Error: tried making a move on a compeleted board.");
        }
        if self.get(row, col).is_some() {
            panic!("Error: tried making a move on an occupied position.");
        }
        self.set(row, col, Some(mark));
        self.check_complete();
        self.check_win();
    }
}

impl fmt::Display for Board {
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

#[cfg(test)]
mod tests {
    use super::*;

    impl Board {
        /// Test helper: Sets an entire row with the provided marks.
        fn set_row(&mut self, row: usize, marks: [Option<Mark>; 3]) {
            for col in 0..3 {
                self.set(row, col, marks[col]);
            }
        }

        /// Test helper: Sets an entire column with the provided marks.
        fn set_col(&mut self, col: usize, marks: [Option<Mark>; 3]) {
            for row in 0..3 {
                self.set(row, col, marks[row]);
            }
        }
    }

    #[test]
    fn test_check_row() {
        let mut board = Board::new();
        assert_eq!(board.check_row(0), None);

        board.set_row(0, [Some(Mark::X), Some(Mark::X), Some(Mark::X)]);
        assert_eq!(board.check_row(0), Some(Mark::X));

        board.set_row(1, [Some(Mark::O), Some(Mark::O), Some(Mark::O)]);
        assert_eq!(board.check_row(1), Some(Mark::O));

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::X)]);
        assert_eq!(board.check_row(0), None);

        board.set(0, 1, Some(Mark::X));
        assert_eq!(board.check_row(0), Some(Mark::X));

        board.set_row(0, [Some(Mark::X), Some(Mark::O), None]);
        assert_eq!(board.check_row(0), None);
    }

    #[test]
    fn test_check_col() {
        let mut board = Board::new();
        assert_eq!(board.check_col(0), None);

        board.set_col(0, [Some(Mark::X), Some(Mark::X), Some(Mark::X)]);
        assert_eq!(board.check_col(0), Some(Mark::X));

        board.set_col(1, [Some(Mark::O), Some(Mark::O), Some(Mark::O)]);
        assert_eq!(board.check_col(1), Some(Mark::O));

        board.set_col(0, [Some(Mark::X), Some(Mark::O), Some(Mark::X)]);
        assert_eq!(board.check_col(0), None);

        board.set(1, 0, Some(Mark::X));
        assert_eq!(board.check_col(0), Some(Mark::X));

        board.set_col(0, [Some(Mark::X), Some(Mark::O), None]);
        assert_eq!(board.check_col(0), None);
    }

    #[test]
    fn test_check_diag() {
        let mut board = Board::new();
        assert_eq!(board.check_diag_dexter(), None);
        assert_eq!(board.check_diag_sinister(), None);

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::O)]);
        board.set_row(1, [Some(Mark::X), None, Some(Mark::X)]);
        board.set_row(2, [Some(Mark::O), Some(Mark::X), Some(Mark::X)]);
        assert_eq!(board.check_diag_dexter(), None);
        assert_eq!(board.check_diag_sinister(), None);

        board.set(1, 1, Some(Mark::X));
        assert_eq!(board.check_diag_dexter(), Some(Mark::X));
        assert_eq!(board.check_diag_sinister(), None);

        board.set(1, 1, Some(Mark::O));
        assert_eq!(board.check_diag_dexter(), None);
        assert_eq!(board.check_diag_sinister(), Some(Mark::O));
    }

    #[test]
    fn test_check_win() {
        let mut board = Board::new();
        assert_eq!(board.check_diag_dexter(), None);
        assert_eq!(board.check_diag_sinister(), None);
        assert_eq!(board.state, GameState::Playing);

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::O)]);
        board.set_row(1, [Some(Mark::X), None, Some(Mark::X)]);
        board.set_row(2, [None, Some(Mark::O), None]);
        assert_eq!(board.check_win(), None);
        assert_eq!(board.state, GameState::Playing);

        board.set(1, 1, Some(Mark::X));
        assert_eq!(board.check_win(), Some(Mark::X));
        assert_eq!(board.state, GameState::Won(Mark::X));

        board.set(1, 1, Some(Mark::O));
        assert_eq!(board.check_win(), Some(Mark::O));
        assert_eq!(board.state, GameState::Won(Mark::O));

        board.set(0, 1, Some(Mark::X));
        assert_eq!(board.check_win(), None);
        board.set(2, 0, Some(Mark::O));
        assert_eq!(board.check_win(), Some(Mark::O));
    }

    #[test]
    fn test_check_draw() {
        let mut board = Board::new();
        assert_eq!(board.check_complete(), false);

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::O)]);
        board.set_row(1, [Some(Mark::X), None, Some(Mark::X)]);
        board.set_row(2, [Some(Mark::O), Some(Mark::O), Some(Mark::X)]);
        assert_eq!(board.check_complete(), false);
        assert_eq!(board.state, GameState::Playing);

        board.set(1, 1, Some(Mark::X));
        assert_eq!(board.check_complete(), true);
        assert_eq!(board.state, GameState::Draw);
    }

    #[test]
    fn test_make_move_win() {
        let mut board = Board::new();

        // Create a winning row for X
        board.make_move(0, 0, Mark::X);
        board.make_move(1, 0, Mark::O);
        board.make_move(0, 1, Mark::X);
        board.make_move(1, 1, Mark::O);
        board.make_move(0, 2, Mark::X);

        assert_eq!(board.state, GameState::Won(Mark::X));
    }

    #[test]
    fn test_make_move_draw() {
        let mut board = Board::new();

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
        let mut board = Board::new();

        board.make_move(0, 0, Mark::X);
        board.make_move(0, 0, Mark::O); // Should panic
    }

    #[test]
    #[should_panic(expected = "tried making a move on a compeleted board")]
    fn test_make_move_on_won_board() {
        let mut board = Board::new();

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
    #[should_panic(expected = "tried making a move on a compeleted board")]
    fn test_make_move_on_draw_board() {
        let mut board = Board::new();

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
        board.make_move(0, 0, Mark::X);
    }
}
