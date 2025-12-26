use crate::ai;
use crate::ai::SimpleAi;
use crate::game::{Board, Mark};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Playing,
    Won(Mark),
    Draw,
}

pub struct App {
    pub board: Board,
    pub active_player: Mark,
    pub state: GameState,
    pub selected_row: usize,
    pub selected_col: usize,
    pub should_quit: bool,
    pub ai: Option<SimpleAi>,
}

impl App {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            active_player: Mark::X,
            state: GameState::Playing,
            selected_row: 0,
            selected_col: 0,
            should_quit: false,
            ai: Some(SimpleAi::new(Mark::O)),
        }
    }

    pub fn input_left(&mut self) {
        for _ in 0..3 {
            // change column
            self.move_selection_left();

            let original_row = self.selected_row;

            // look for free positions in the current column
            for _ in 0..3 {
                if self
                    .board
                    .get(self.selected_row, self.selected_col)
                    .is_none()
                {
                    return;
                }
                match original_row {
                    0 => self.move_selection_down(),
                    2 => self.move_selection_up(),
                    _ => self.move_selection_up(),
                }
            }
        }
    }

    pub fn input_right(&mut self) {
        for _ in 0..3 {
            // change column
            self.move_selection_right();

            let original_row = self.selected_row;

            // look for free positions in the current column
            for _ in 0..3 {
                if self
                    .board
                    .get(self.selected_row, self.selected_col)
                    .is_none()
                {
                    return;
                }
                match original_row {
                    0 => self.move_selection_down(),
                    2 => self.move_selection_up(),
                    _ => self.move_selection_down(),
                }
            }
        }
    }

    pub fn input_up(&mut self) {
        for _ in 0..3 {
            // change row
            self.move_selection_up();

            let original_col = self.selected_col;

            // look for free positions in the current row
            for _ in 0..3 {
                if self
                    .board
                    .get(self.selected_row, self.selected_col)
                    .is_none()
                {
                    return;
                }
                match original_col {
                    0 => self.move_selection_right(),
                    2 => self.move_selection_left(),
                    _ => self.move_selection_left(),
                }
            }
        }
    }

    pub fn input_down(&mut self) {
        for _ in 0..3 {
            // change row
            self.move_selection_down();

            let original_col = self.selected_col;

            // look for free positions in the current row
            for _ in 0..3 {
                if self
                    .board
                    .get(self.selected_row, self.selected_col)
                    .is_none()
                {
                    return;
                }
                match original_col {
                    0 => self.move_selection_right(),
                    2 => self.move_selection_left(),
                    _ => self.move_selection_right(),
                }
            }
        }
    }

    fn move_selection_left(&mut self) {
        if self.selected_col > 0 {
            self.selected_col -= 1;
        } else {
            self.selected_col = 2;
        }
    }

    fn move_selection_right(&mut self) {
        self.selected_col = (self.selected_col + 1) % 3;
    }

    fn move_selection_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
        } else {
            self.selected_row = 2;
        }
    }

    fn move_selection_down(&mut self) {
        self.selected_row = (self.selected_row + 1) % 3;
    }

    fn move_selection_next_available(&mut self) {
        self.selected_col += 1;
        if self.selected_col >= 3 {
            self.selected_col = 0;
            self.selected_row += 1;
            if self.selected_row >= 3 {
                self.selected_row = 0;
            }
        }
    }

    pub fn make_move(&mut self) {
        if self.state != GameState::Playing {
            return;
        }

        // Check if cell is empty
        if self
            .board
            .get(self.selected_row, self.selected_col)
            .is_some()
        {
            return;
        }

        // Make the move
        self.board.set(
            self.selected_row,
            self.selected_col,
            Some(self.active_player),
        );

        // Check for win
        if let Some(winner) = self.board.check_all() {
            self.state = GameState::Won(winner);
            return;
        }

        // Check for draw
        if self.board.check_complete() {
            self.state = GameState::Draw;
            return;
        }

        // Switch player
        self.active_player = match self.active_player {
            Mark::X => Mark::O,
            Mark::O => Mark::X,
        };

        // let AI play
        if let Some(ai) = &self.ai {
            let (ai_row, ai_col) = ai.choose_move(self.board.clone());
            self.board.set(ai_row, ai_col, Some(ai.ai_mark));
        }

        // Check for win
        if let Some(winner) = self.board.check_all() {
            self.state = GameState::Won(winner);
            return;
        }

        // Check for draw
        if self.board.check_complete() {
            self.state = GameState::Draw;
            return;
        }

        // Switch player
        self.active_player = match self.active_player {
            Mark::X => Mark::O,
            Mark::O => Mark::X,
        };

        // Reset position
        (self.selected_row, self.selected_col) = (0, 0);
        while self
            .board
            .get(self.selected_row, self.selected_col)
            .is_some()
        {
            self.move_selection_next_available();
        }
    }

    pub fn reset(&mut self) {
        self.board = Board::new();
        self.active_player = Mark::X;
        self.state = GameState::Playing;
        self.selected_row = 0;
        self.selected_col = 0;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
