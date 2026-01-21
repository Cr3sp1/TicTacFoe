pub mod base;
pub mod ultimate;

use std::fmt;

/// Represents a player's mark (X or O) on the tic-tac-toe board.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mark {
    X,
    O,
}

impl fmt::Display for Mark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mark::X => write!(f, "X"),
            Mark::O => write!(f, "O"),
        }
    }
}

/// Represents the current state of a tic-tac-toe game.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameState {
    /// The game is still in progress.
    Playing,
    /// The game has been won by the specified mark.
    Won(Mark),
    /// The game ended in a draw.
    Draw,
}

/// Trait for types that can act as a tic-tac-toe board.
///
/// Implementors must provide a method to get the mark at a specific position.
pub trait Board {
    /// Gets the mark at the specified position.
    ///
    /// # Arguments
    /// * `row` - Row index (0-2)
    /// * `col` - Column index (0-2)
    ///
    /// # Returns
    /// The mark at the position, or None if the cell is empty.
    fn get(&self, row: usize, col: usize) -> Option<Mark>;
}

/// Checks if the specified row has three matching marks.
///
/// # Arguments
/// * `board` - A reference to any type implementing the Board trait
/// * `row` - Row index (0-2)
///
/// # Returns
/// The winning mark if all three cells in the row match,
/// or None if they don't match or any cell is empty.
pub fn check_row(board: &impl Board, row: usize) -> Option<Mark> {
    let mark_0 = board.get(row, 0)?;
    for i in 1..3 {
        let mark_i = board.get(row, i)?;
        if mark_i != mark_0 {
            return None;
        }
    }
    Some(mark_0)
}

/// Checks if the specified column has three matching marks.
///
/// # Arguments
/// * `board` - A reference to any type implementing the Board trait
/// * `col` - Column index (0-2)
///
/// # Returns
/// The winning mark if all three cells in the column match,
/// or None if they don't match or any cell is empty.
pub fn check_col(board: &impl Board, col: usize) -> Option<Mark> {
    let mark_0 = board.get(0, col)?;
    for i in 1..3 {
        let mark_i = board.get(i, col)?;
        if mark_i != mark_0 {
            return None;
        }
    }
    Some(mark_0)
}

/// Checks the top-left to bottom-right diagonal for three matching marks.
///
/// # Arguments
/// * `board` - A reference to any type implementing the Board trait
///
/// # Returns
/// The winning mark if all three cells match, or None otherwise.
pub fn check_diag_dexter(board: &impl Board) -> Option<Mark> {
    let mark_0 = board.get(0, 0)?;
    for i in 1..3 {
        let mark_i = board.get(i, i)?;
        if mark_i != mark_0 {
            return None;
        }
    }
    Some(mark_0)
}

/// Checks the top-right to bottom-left diagonal for three matching marks.
///
/// # Arguments
/// * `board` - A reference to any type implementing the Board trait
///
/// # Returns
/// The winning mark if all three cells match, or None otherwise.
pub fn check_diag_sinister(board: &impl Board) -> Option<Mark> {
    let mark_0 = board.get(0, 2)?;
    for i in 1..3 {
        let mark_i = board.get(i, 2 - i)?;
        if mark_i != mark_0 {
            return None;
        }
    }
    Some(mark_0)
}

/// Checks all possible winning conditions (rows, columns, and diagonals).
///
/// # Arguments
/// * `board` - A reference to any type implementing the Board trait
///
/// # Returns
/// The winning mark if any winning condition is met, or None if
/// there is no winner yet.
pub fn check_win(board: &impl Board) -> Option<Mark> {
    if let Some(mark) = check_diag_dexter(board) {
        return Some(mark);
    }
    if let Some(mark) = check_diag_sinister(board) {
        return Some(mark);
    }
    for i in 0..3 {
        if let Some(mark) = check_row(board, i) {
            return Some(mark);
        }
        if let Some(mark) = check_col(board, i) {
            return Some(mark);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::base::*;
    use super::*;

    #[test]
    fn test_check_row() {
        let mut board = SmallBoard::new();
        assert_eq!(check_row(&board, 0), None);

        board.set_row(0, [Some(Mark::X), Some(Mark::X), Some(Mark::X)]);
        assert_eq!(check_row(&board, 0), Some(Mark::X));

        board.set_row(1, [Some(Mark::O), Some(Mark::O), Some(Mark::O)]);
        assert_eq!(check_row(&board, 1), Some(Mark::O));

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::X)]);
        assert_eq!(check_row(&board, 0), None);

        board.set(0, 1, Some(Mark::X));
        assert_eq!(check_row(&board, 0), Some(Mark::X));

        board.set_row(0, [Some(Mark::X), Some(Mark::O), None]);
        assert_eq!(check_row(&board, 0), None);
    }

    #[test]
    fn test_check_col() {
        let mut board = SmallBoard::new();
        assert_eq!(check_col(&board, 0), None);

        board.set_col(0, [Some(Mark::X), Some(Mark::X), Some(Mark::X)]);
        assert_eq!(check_col(&board, 0), Some(Mark::X));

        board.set_col(1, [Some(Mark::O), Some(Mark::O), Some(Mark::O)]);
        assert_eq!(check_col(&board, 1), Some(Mark::O));

        board.set_col(0, [Some(Mark::X), Some(Mark::O), Some(Mark::X)]);
        assert_eq!(check_col(&board, 0), None);

        board.set(1, 0, Some(Mark::X));
        assert_eq!(check_col(&board, 0), Some(Mark::X));

        board.set_col(0, [Some(Mark::X), Some(Mark::O), None]);
        assert_eq!(check_col(&board, 0), None);
    }

    #[test]
    fn test_check_diag() {
        let mut board = SmallBoard::new();
        assert_eq!(check_diag_dexter(&board), None);
        assert_eq!(check_diag_sinister(&board), None);

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::O)]);
        board.set_row(1, [Some(Mark::X), None, Some(Mark::X)]);
        board.set_row(2, [Some(Mark::O), Some(Mark::X), Some(Mark::X)]);
        assert_eq!(check_diag_dexter(&board), None);
        assert_eq!(check_diag_sinister(&board), None);

        board.set(1, 1, Some(Mark::X));
        assert_eq!(check_diag_dexter(&board), Some(Mark::X));
        assert_eq!(check_diag_sinister(&board), None);

        board.set(1, 1, Some(Mark::O));
        assert_eq!(check_diag_dexter(&board), None);
        assert_eq!(check_diag_sinister(&board), Some(Mark::O));
    }

    #[test]
    fn test_check_win() {
        let mut board = SmallBoard::new();
        assert_eq!(check_diag_dexter(&board), None);
        assert_eq!(check_diag_sinister(&board), None);

        board.set_row(0, [Some(Mark::X), Some(Mark::O), Some(Mark::O)]);
        board.set_row(1, [Some(Mark::X), None, Some(Mark::X)]);
        board.set_row(2, [None, Some(Mark::O), None]);
        assert_eq!(check_win(&board), None);
        assert_eq!(board.state, GameState::Playing);

        board.set(1, 1, Some(Mark::X));
        assert_eq!(check_win(&board), Some(Mark::X));

        board.set(1, 1, Some(Mark::O));
        assert_eq!(check_win(&board), Some(Mark::O));

        board.set(0, 1, Some(Mark::X));
        assert_eq!(check_win(&board), None);
        board.set(2, 0, Some(Mark::O));
        assert_eq!(check_win(&board), Some(Mark::O));
    }
}
