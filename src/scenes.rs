use crate::ai::SimpleAi;
use crate::game::{Board, Mark};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    PvE,
    LocalPvP,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Playing,
    Won(Mark),
    Draw,
}

pub struct MainMenu {
    pub selected_option: usize,
    pub options: Vec<&'static str>,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            selected_option: 0,
            options: vec!["Local PvP", "Play vs AI", "Quit"],
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_option > 0 {
            self.selected_option -= 1;
        } else {
            self.selected_option = self.options.len() - 1;
        }
    }

    pub fn move_down(&mut self) {
        self.selected_option = (self.selected_option + 1) % self.options.len();
    }

    pub fn get_selected(&self) -> &'static str {
        self.options[self.selected_option]
    }
}

pub struct GamePlay {
    pub board: Board,
    pub active_player: Mark,
    pub state: GameState,
    pub turn: u32,
    pub mode: GameMode,
    pub selected_row: usize,
    pub selected_col: usize,
    pub ai: Option<SimpleAi>,
}

impl GamePlay {
    pub fn new(mode: GameMode) -> Self {
        let ai = if mode == GameMode::PvE {
            Some(SimpleAi::new(Mark::O))
        } else {
            None
        };

        Self {
            board: Board::new(),
            active_player: Mark::X,
            state: GameState::Playing,
            turn: 0,
            mode,
            selected_row: 0,
            selected_col: 0,
            ai,
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

        self.turn += 1;

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

        if self.mode == GameMode::PvE {
            self.ai_play();
        }

        self.reset_position();
    }

    fn ai_play(&mut self) {
        if let Some(ai) = &self.ai {
            let (ai_row, ai_col) = ai.choose_move(self.board.clone());
            self.board.set(ai_row, ai_col, Some(ai.ai_mark));

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
            self.active_player = match ai.ai_mark {
                Mark::X => Mark::O,
                Mark::O => Mark::X,
            };

            self.turn += 1;

            self.reset_position();
        }
    }

    pub fn play_second(&mut self) {
        if self.state == GameState::Playing && self.turn == 0 {
            self.ai_play();
        }
    }

    fn reset_position(&mut self) {
        if self.state == GameState::Draw {
            return;
        }
        (self.selected_row, self.selected_col) = (0, 0);
        while self
            .board
            .get(self.selected_row, self.selected_col)
            .is_some()
        {
            self.move_selection_next_available();
        }
    }

    pub fn reset_game(&mut self) {
        self.board = Board::new();
        self.active_player = Mark::X;
        self.state = GameState::Playing;
        self.turn = 0;
        self.selected_row = 0;
        self.selected_col = 0;
    }
}
