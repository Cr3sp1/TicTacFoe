use crate::ai::SimpleAi;
use crate::game::base::SmallBoard;
use crate::game::{Board, GameState, Mark};

/// Represents the game mode selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    PvE,
    LocalPvP,
}

/// Main menu scene with selectable options.
pub struct MainMenu {
    pub selected_option: usize,
    pub options: Vec<&'static str>,
}

impl MainMenu {
    /// Creates a new main menu with default options.
    pub fn new() -> Self {
        Self {
            selected_option: 0,
            options: vec!["Local PvP", "Play vs AI", "Quit"],
        }
    }

    /// Moves selection up, wrapping to bottom if at top.
    pub fn move_up(&mut self) {
        if self.selected_option > 0 {
            self.selected_option -= 1;
        } else {
            self.selected_option = self.options.len() - 1;
        }
    }

    /// Moves selection down, wrapping to top if at bottom.
    pub fn move_down(&mut self) {
        self.selected_option = (self.selected_option + 1) % self.options.len();
    }

    /// Returns the currently selected option.
    pub fn get_selected(&self) -> &'static str {
        self.options[self.selected_option]
    }
}

/// Main gameplay scene containing the board state and game logic.
pub struct GamePlay {
    pub board: SmallBoard,
    pub active_player: Mark,
    pub turn: u32,
    pub mode: GameMode,
    pub selected_row: usize,
    pub selected_col: usize,
    pub ai: Option<SimpleAi>,
}

impl GamePlay {
    /// Creates a new game with the specified mode.
    ///
    /// For PvE mode, initializes an AI opponent playing as O.
    pub fn new(mode: GameMode) -> Self {
        let ai = if mode == GameMode::PvE {
            Some(SimpleAi::new(Mark::O))
        } else {
            None
        };

        Self {
            board: SmallBoard::new(),
            active_player: Mark::X,
            turn: 0,
            mode,
            selected_row: 0,
            selected_col: 0,
            ai,
        }
    }

    /// Moves selection left, wrapping to the rightmost column and finding
    /// the next available cell if the target is occupied.
    pub fn input_left(&mut self) {
        for _ in 0..3 {
            self.move_selection_left();
            let original_row = self.selected_row;

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

    /// Moves selection right, wrapping to the leftmost column and finding
    /// the next available cell if the target is occupied.
    pub fn input_right(&mut self) {
        for _ in 0..3 {
            self.move_selection_right();
            let original_row = self.selected_row;

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

    /// Moves selection up, wrapping to the bottom row and finding
    /// the next available cell if the target is occupied.
    pub fn input_up(&mut self) {
        for _ in 0..3 {
            self.move_selection_up();
            let original_col = self.selected_col;

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

    /// Moves selection down, wrapping to the top row and finding
    /// the next available cell if the target is occupied.
    pub fn input_down(&mut self) {
        for _ in 0..3 {
            self.move_selection_down();
            let original_col = self.selected_col;

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

    /// Attempts to make a move at the currently selected position.
    ///
    /// If the game is over or the cell is occupied, does nothing.
    /// After a valid move, checks for win/draw conditions and switches players.
    /// In PvE mode, triggers the AI to make its move.
    pub fn player_move(&mut self) {
        if self.board.state != GameState::Playing {
            return;
        }

        self.board
            .make_move(self.selected_row, self.selected_col, self.active_player);

        self.turn += 1;

        if self.board.state != GameState::Playing {
            return;
        }

        self.active_player = match self.active_player {
            Mark::X => Mark::O,
            Mark::O => Mark::X,
        };

        if self.mode == GameMode::PvE {
            self.ai_play();
        }

        self.reset_position();
    }

    /// Executes the AI's turn in PvE mode.
    fn ai_play(&mut self) {
        if let Some(ai) = &self.ai {
            let (ai_row, ai_col) = ai.choose_move(self.board.clone());
            self.board.make_move(ai_row, ai_col, ai.ai_mark);

            self.active_player = match ai.ai_mark {
                Mark::X => Mark::O,
                Mark::O => Mark::X,
            };

            self.turn += 1;

            self.reset_position();
        } else {
            panic!("Error: no AI is available.");
        }
    }

    /// Allows the AI to play first if the game just started.
    pub fn play_second(&mut self) {
        if self.board.state == GameState::Playing && self.turn == 0 {
            self.ai_play();
        }
    }

    /// Resets the selected position to the first available cell.
    fn reset_position(&mut self) {
        if self.board.state == GameState::Draw {
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

    /// Resets the game to initial state while keeping the same mode.
    pub fn reset_game(&mut self) {
        self.board = SmallBoard::new();
        self.active_player = Mark::X;
        self.turn = 0;
        self.selected_row = 0;
        self.selected_col = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_menu_new() {
        let menu = MainMenu::new();
        assert_eq!(menu.selected_option, 0);
        assert_eq!(menu.options.len(), 3);
        assert_eq!(menu.get_selected(), "Local PvP");
    }

    #[test]
    fn test_main_menu_move_down() {
        let mut menu = MainMenu::new();
        menu.move_down();
        assert_eq!(menu.selected_option, 1);
        assert_eq!(menu.get_selected(), "Play vs AI");
    }

    #[test]
    fn test_main_menu_move_up_wraps() {
        let mut menu = MainMenu::new();
        menu.move_up();
        assert_eq!(menu.selected_option, 2);
        assert_eq!(menu.get_selected(), "Quit");
    }

    #[test]
    fn test_main_menu_move_down_wraps() {
        let mut menu = MainMenu::new();
        menu.selected_option = 2;
        menu.move_down();
        assert_eq!(menu.selected_option, 0);
    }

    #[test]
    fn test_gameplay_new_pvp() {
        let game = GamePlay::new(GameMode::LocalPvP);
        assert_eq!(game.mode, GameMode::LocalPvP);
        assert_eq!(game.active_player, Mark::X);
        assert_eq!(game.board.state, GameState::Playing);
        assert_eq!(game.turn, 0);
        assert!(game.ai.is_none());
    }

    #[test]
    fn test_gameplay_new_pve() {
        let game = GamePlay::new(GameMode::PvE);
        assert_eq!(game.mode, GameMode::PvE);
        assert!(game.ai.is_some());
    }

    #[test]
    fn test_player_move_places_mark() {
        let mut game = GamePlay::new(GameMode::LocalPvP);
        game.player_move();

        assert!(game.board.get(0, 0).is_some());
        assert_eq!(game.board.get(0, 0).unwrap(), Mark::X);
        assert_eq!(game.turn, 1);
    }

    #[test]
    fn test_player_move_switches_player() {
        let mut game = GamePlay::new(GameMode::LocalPvP);
        assert_eq!(game.active_player, Mark::X);

        game.player_move();
        assert_eq!(game.active_player, Mark::O);

        game.input_right();
        game.player_move();
        assert_eq!(game.active_player, Mark::X);
    }

    #[test]
    fn test_player_move_detects_win() {
        let mut game = GamePlay::new(GameMode::LocalPvP);
        // Set up winning position for X
        game.board.make_move(0, 0, Mark::X);
        game.board.make_move(0, 1, Mark::X);
        game.selected_row = 0;
        game.selected_col = 2;

        game.player_move();

        assert_eq!(game.board.state, GameState::Won(Mark::X));
    }

    #[test]
    fn test_reset_game() {
        let mut game = GamePlay::new(GameMode::LocalPvP);
        game.player_move();
        game.input_right();
        game.player_move();

        game.reset_game();

        assert_eq!(game.turn, 0);
        assert_eq!(game.active_player, Mark::X);
        assert_eq!(game.board.state, GameState::Playing);
        assert!(game.board.get(0, 0).is_none());
    }

    #[test]
    fn test_selection_wraps_horizontally() {
        let mut game = GamePlay::new(GameMode::LocalPvP);
        game.selected_col = 2;

        game.move_selection_right();
        assert_eq!(game.selected_col, 0);

        game.move_selection_left();
        assert_eq!(game.selected_col, 2);
    }

    #[test]
    fn test_selection_wraps_vertically() {
        let mut game = GamePlay::new(GameMode::LocalPvP);
        game.selected_row = 2;

        game.move_selection_down();
        assert_eq!(game.selected_row, 0);

        game.move_selection_up();
        assert_eq!(game.selected_row, 2);
    }
}
