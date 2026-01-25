use super::base::SmallBoard;
use super::*;

/// A 3x3 grid of tic-tac-toe boards for Ultimate Tic-Tac-Toe.
///
/// The board is represented as a flat array of 9 small boards.
#[derive(Copy, Clone)]
pub struct BigBoard {
    boards: [SmallBoard; 9],
    pub state: GameState,
    pub active_board: Option<(usize, usize)>,
}

impl BigBoard {
    /// Creates a new BigBoard with all small boards empty.
    pub fn new() -> Self {
        BigBoard {
            boards: [SmallBoard::new(); 9],
            state: GameState::Playing,
            active_board: None,
        }
    }

    /// Gets a reference to the small board at the specified position.
    ///
    /// # Arguments
    /// * `board_row` - Row index of the small board (0-2)
    /// * `board_col` - Column index of the small board (0-2)
    ///
    /// # Panics
    /// Panics if board_row or board_col is greater than 3.
    pub fn get_board(&self, board_row: usize, board_col: usize) -> &SmallBoard {
        if board_row > 3 || board_col > 3 {
            panic!(
                "Error: tried to access board({board_row}, {board_col}) which is out of bounds."
            );
        }
        &self.boards[board_row * 3 + board_col]
    }

    /// Gets the mark at the specified position within a specific small board.
    ///
    /// # Arguments
    /// * `board_row` - Row index of the small board (0-2)
    /// * `board_col` - Column index of the small board (0-2)
    /// * `cell_row` - Row index within the small board (0-2)
    /// * `cell_col` - Column index within the small board (0-2)
    ///
    /// # Returns
    /// The mark at the specified position, or None if the cell is empty.
    ///
    /// # Panics
    /// Panics if any index is greater than 3.
    pub fn get(
        &mut self,
        board_row: usize,
        board_col: usize,
        cell_row: usize,
        cell_col: usize,
    ) -> Option<Mark> {
        self.get_board(board_row, board_col).get(cell_row, cell_col)
    }

    /// Checks if all small boards are either won or complete (draw).
    ///
    /// # Returns
    /// True if every small board's state is not GameState::Playing, false otherwise.
    pub fn check_complete(&self) -> bool {
        for board_row in 0..3 {
            for board_col in 0..3 {
                if self.boards[board_row * 3 + board_col].state == GameState::Playing {
                    return false;
                }
            }
        }
        true
    }

    /// Makes a move on the BigBoard at the specified position.
    ///
    /// Places the given mark in the specified small board at the specified
    /// row and column, then checks if the move resulted in a win or draw
    /// and updates the game state accordingly.
    ///
    /// # Arguments
    /// * `board_row` - Row index of the small board (0-2)
    /// * `board_col` - Column index of the small board (0-2)
    /// * `cell_row` - Row index within the small board (0-2)
    /// * `cell_col` - Column index within the small board (0-2)
    /// * `mark` - The mark to place (Mark::X or Mark::O)
    ///
    /// # Panics
    /// * Panics if the BigBoard game is already over (state is not GameState::Playing)
    /// * Panics if there is an active board constraint and the move is attempted on a different board
    /// * Panics if the specified position is already occupied (delegated to SmallBoard::make_move)
    pub fn make_move(
        &mut self,
        board_row: usize,
        board_col: usize,
        cell_row: usize,
        cell_col: usize,
        mark: Mark,
    ) {
        if self.state != GameState::Playing {
            panic!("Error: tried making a move on a compeleted big board.");
        }
        if let Some(active_board) = self.active_board {
            if (board_row, board_col) != active_board {
                panic!("Error: tried making a move on a board different than the active board.");
            }
        }

        self.boards[board_row * 3 + board_col].make_move(cell_row, cell_col, mark);
        if self.check_complete() {
            self.state = GameState::Draw;
        }
        if let Some(mark) = check_win(self) {
            self.state = GameState::Won(mark);
        };

        self.active_board = match self.get_board(cell_row, cell_col).state {
            GameState::Playing => Some((cell_row, cell_col)),
            _ => None,
        }
    }
}

impl Board for BigBoard {
    /// Gets the winning mark for a small board at the specified position.
    ///
    /// This implementation of the Board trait treats each small board as a single
    /// cell in the meta-game. It returns the winning mark if the small board has
    /// been won, or None if the board is still in play or ended in a draw.
    ///
    /// # Arguments
    /// * `board_row` - Row index of the small board (0-2)
    /// * `board_col` - Column index of the small board (0-2)
    ///
    /// # Returns
    /// The winning mark if the small board at this position has been won,
    /// or None if it's still playing or ended in a draw.
    ///
    /// # Panics
    /// Panics if board_row or board_col is greater than 3.
    fn get(&self, board_row: usize, board_col: usize) -> Option<Mark> {
        if let GameState::Won(mark) = self.get_board(board_row, board_col).state {
            Some(mark)
        } else {
            None
        }
    }

    /// Gets whether it is possible to play in the specified position.
    ///
    /// # Arguments
    /// * `row` - Row index (0-2)
    /// * `col` - Column index (0-2)
    ///
    /// # Returns
    /// True if the board is not complete, else False.
    fn is_playable(&self, row: usize, col: usize) -> bool {
        self.get_board(row, col).state == GameState::Playing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_complete() {
        let mut board = BigBoard::new();

        assert_eq!(board.check_complete(), false);

        // Win the first small board
        board.boards[0].make_move(0, 0, Mark::X);
        board.boards[0].make_move(1, 0, Mark::O);
        board.boards[0].make_move(0, 1, Mark::X);
        board.boards[0].make_move(1, 1, Mark::O);
        board.boards[0].make_move(0, 2, Mark::X);

        assert_eq!(board.boards[0].state, GameState::Won(Mark::X));
        assert_eq!(board.check_complete(), false);

        // Win all other 8 small boards
        for i in 1..9 {
            board.boards[i].make_move(0, 0, Mark::X);
            board.boards[i].make_move(1, 0, Mark::O);
            board.boards[i].make_move(0, 1, Mark::X);
            board.boards[i].make_move(1, 1, Mark::O);
            board.boards[i].make_move(0, 2, Mark::X);
        }

        assert_eq!(board.check_complete(), true);
    }

    #[test]
    fn test_make_move_win_board_board() {
        let mut board = BigBoard::new();

        // Play a couple moves
        board.make_move(0, 0, 0, 0, Mark::X);
        assert_eq!(board.active_board, Some((0, 0)));
        board.make_move(0, 0, 1, 0, Mark::O);
        assert_eq!(board.active_board, Some((1, 0)));
        board.active_board = None;

        // Win board (0, 0)
        board.boards[0].make_move(0, 1, Mark::X);
        board.make_move(0, 0, 0, 2, Mark::X);
        assert_eq!(board.boards[0].state, GameState::Won(Mark::X));
        assert_eq!(board.state, GameState::Playing);
        assert_eq!(board.active_board, Some((0, 2)));
        board.active_board = None;

        // Check that active board gets set to None if target board is complete
        board.make_move(2, 2, 0, 0, Mark::X);
        assert!(board.active_board.is_none());

        // Win board (0, 1)
        board.boards[1].make_move(0, 0, Mark::X);
        board.boards[1].make_move(1, 0, Mark::O);
        board.boards[1].make_move(0, 1, Mark::X);
        board.boards[1].make_move(1, 1, Mark::O);
        board.boards[1].make_move(0, 2, Mark::X);

        // Win board (0, 2) - this should win the big board
        board.boards[2].make_move(0, 0, Mark::X);
        board.boards[2].make_move(1, 0, Mark::O);
        board.boards[2].make_move(0, 1, Mark::X);
        board.boards[2].make_move(1, 1, Mark::O);
        board.make_move(0, 2, 0, 2, Mark::X);

        assert_eq!(board.state, GameState::Won(Mark::X));
    }

    #[test]
    fn test_make_move_draw_board_board() {
        let mut board = BigBoard::new();

        // Create a scenario where all boards are complete but no one wins
        for i in 0..9 {
            // Create a draw in each small board
            board.boards[i].make_move(0, 0, Mark::X);
            board.boards[i].make_move(0, 1, Mark::O);
            board.boards[i].make_move(0, 2, Mark::X);
            board.boards[i].make_move(1, 0, Mark::X);
            board.boards[i].make_move(1, 1, Mark::O);
            board.boards[i].make_move(1, 2, Mark::O);
            board.boards[i].make_move(2, 0, Mark::O);
            board.boards[i].make_move(2, 1, Mark::X);
            if i < 8 {
                board.boards[i].make_move(2, 2, Mark::X);
            }
        }
        board.make_move(2, 2, 2, 2, Mark::X);

        assert_eq!(board.state, GameState::Draw);
    }

    #[test]
    #[should_panic(expected = "tried making a move on a board different than the active board")]
    fn test_make_move_wrong_active_board() {
        let mut board = BigBoard::new();
        board.active_board = Some((0, 0));

        // Try to make a move on a different board
        board.make_move(1, 1, 0, 0, Mark::X); // Should panic
    }

    #[test]
    #[should_panic(expected = "tried making a move on an occupied position")]
    fn test_make_move_occupied_position() {
        let mut board = BigBoard::new();

        board.make_move(0, 0, 0, 0, Mark::X);
        board.make_move(0, 0, 0, 0, Mark::O); // Should panic
    }
}
