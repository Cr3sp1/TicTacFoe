use crate::ai::SimpleAi;
use crate::game::base::SmallBoard;
use crate::game::ultimate::BigBoard;
use crate::game::{GameState, Mark};
use crate::utils::{
    Position, move_selection_down_playable, move_selection_left_playable,
    move_selection_right_playable, move_selection_up_playable, reset_position,
};

pub const MAIN_MENU_OPTIONS: [&'static str; 3] = ["Ultimate Tic Tac Toe", "Tic Tac Toe", "Quit"];
pub const TTT_MENU_OPTIONS: [&'static str; 3] = ["Local PvP", "Play vs AI", "Back"];
pub const UTT_MENU_OPTIONS: [&'static str; 2] = ["Local PvP", "Back"];

/// Represents all the possible scenes.
pub enum Scene {
    MainMenu(Menu),
    TTTMenu(Menu),
    UTTMenu(Menu),
    PlayingTTT(GamePlayTTT),
    PlayingUTT(GamePlayUTT),
}

/// Represents the game mode selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    PvE,
    LocalPvP,
}

/// Menu scene with selectable options.
pub struct Menu {
    pub selected_option: usize,
    pub options: Vec<&'static str>,
}

impl Menu {
    /// Creates a new menu.
    pub fn new(options: Vec<&'static str>) -> Self {
        Self {
            selected_option: 0,
            options,
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

/// Main tic-tac-toe gameplay scene containing the board state and game logic.
pub struct GamePlayTTT {
    pub board: SmallBoard,
    pub active_player: Mark,
    pub turn: u32,
    pub mode: GameMode,
    pub selected: Position,
    pub ai: Option<SimpleAi>,
}

impl GamePlayTTT {
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
            selected: Position { row: 0, col: 0 },
            ai,
        }
    }

    /// Moves selection left, wrapping to the rightmost column and finding
    /// the next available cell if the target is occupied.
    pub fn input_left(&mut self) {
        move_selection_left_playable(&self.board, &mut self.selected);
    }

    /// Moves selection right, wrapping to the leftmost column and finding
    /// the next available cell if the target is occupied.
    pub fn input_right(&mut self) {
        move_selection_right_playable(&self.board, &mut self.selected);
    }

    /// Moves selection up, wrapping to the bottom row and finding
    /// the next available cell if the target is occupied.
    pub fn input_up(&mut self) {
        move_selection_up_playable(&self.board, &mut self.selected);
    }

    /// Moves selection down, wrapping to the top row and finding
    /// the next available cell if the target is occupied.
    pub fn input_down(&mut self) {
        move_selection_down_playable(&self.board, &mut self.selected);
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
            .make_move(self.selected.row, self.selected.col, self.active_player);

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

        reset_position(&self.board, &mut self.selected);
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

            reset_position(&self.board, &mut self.selected);
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

    /// Resets the game to initial state while keeping the same mode.
    pub fn reset_game(&mut self) {
        self.board = SmallBoard::new();
        self.active_player = Mark::X;
        self.turn = 0;
        self.selected.row = 0;
        self.selected.col = 0;
    }
}

/// Main tic-tac-toe gameplay scene containing the board state and game logic.
pub struct GamePlayUTT {
    pub big_board: BigBoard,
    pub active_player: Mark,
    pub turn: u32,
    pub mode: GameMode,
    pub selected_board: Position,
    pub selected_cell: Option<Position>,
    // pub ai: Option<SimpleAi>,
}

impl GamePlayUTT {
    /// Creates a new game with the specified mode.
    ///
    /// For PvE mode, initializes an AI opponent playing as O.
    pub fn new(mode: GameMode) -> Self {
        Self {
            big_board: BigBoard::new(),
            active_player: Mark::X,
            turn: 0,
            mode,
            selected_board: Position { row: 0, col: 0 },
            selected_cell: None,
        }
    }

    /// Apply move function to selected cell if it exists, else apply it to selected board
    fn input_move(
        &mut self,
        f_small: fn(&SmallBoard, &mut Position),
        f_big: fn(&BigBoard, &mut Position),
    ) {
        let Self {
            big_board,
            selected_board,
            selected_cell,
            ..
        } = self;
        if let Some(cell) = selected_cell {
            let selected_board = big_board.get_board(selected_board.row, selected_board.col);
            f_small(selected_board, cell);
        } else {
            f_big(&self.big_board, &mut self.selected_board);
        }
    }

    /// Moves selected cell left if it exists, else moves the selected board
    pub fn input_left(&mut self) {
        self.input_move(move_selection_left_playable, move_selection_left_playable);
    }

    /// Moves selected cell right if it exists, else moves the selected board
    pub fn input_right(&mut self) {
        self.input_move(move_selection_right_playable, move_selection_right_playable);
    }

    /// Moves selected cell up if it exists, else moves the selected board
    pub fn input_up(&mut self) {
        self.input_move(move_selection_up_playable, move_selection_up_playable);
    }

    /// Moves selected cell down if it exists, else moves the selected board
    pub fn input_down(&mut self) {
        self.input_move(move_selection_down_playable, move_selection_down_playable);
    }

    /// Selects the board if not selected already, else plays the move
    pub fn input_enter(&mut self) {
        if self.selected_cell.is_none() {
            let mut cell_position = Position { row: 0, col: 0 };
            let selected_board = self
                .big_board
                .get_board(self.selected_board.row, self.selected_board.col);
            reset_position(selected_board, &mut cell_position);
            self.selected_cell = Some(cell_position);
        } else {
            self.player_move();
        }
    }

    /// Deselects the board if possible
    pub fn input_esc(&mut self) {
        if self.big_board.active_board.is_none() {
            self.selected_cell = None;
        }
    }

    /// Attempts to make a move at the currently selected board and cell.
    ///
    /// If the game is over does nothing. If there is no selected cell it panics.
    /// After a valid move, checks for win/draw conditions and switches players.
    /// In PvE mode, triggers the AI to make its move.
    pub fn player_move(&mut self) {
        if self.big_board.state != GameState::Playing {
            return;
        }

        let selected_cell = self.selected_cell.as_mut().unwrap();
        self.big_board.make_move(
            self.selected_board.row,
            self.selected_board.col,
            selected_cell.row,
            selected_cell.col,
            self.active_player,
        );

        self.turn += 1;

        if self.big_board.state != GameState::Playing {
            return;
        }

        self.active_player = match self.active_player {
            Mark::X => Mark::O,
            Mark::O => Mark::X,
        };

        // If next action is constrained to active board, select it by default
        if let Some((active_row, active_col)) = self.big_board.active_board {
            self.selected_board = Position {
                row: active_row,
                col: active_col,
            };
            reset_position(
                self.big_board.get_board(active_row, active_col),
                selected_cell,
            );
        } else {
            reset_position(&self.big_board, &mut self.selected_board);
            self.selected_cell = None;
        }
    }

    /// Resets the game to initial state while keeping the same mode.
    pub fn reset_game(&mut self) {
        self.big_board = BigBoard::new();
        self.active_player = Mark::X;
        self.turn = 0;
        self.selected_board = Position { row: 0, col: 0 };
        self.selected_cell = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Board;

    #[test]
    fn test_main_menu_new() {
        let menu = Menu::new(TTT_MENU_OPTIONS.to_vec());
        assert_eq!(menu.selected_option, 0);
        assert_eq!(menu.options.len(), 3);
        assert_eq!(menu.get_selected(), "Local PvP");
    }

    #[test]
    fn test_menu_move_down() {
        let mut menu = Menu::new(TTT_MENU_OPTIONS.to_vec());
        menu.move_down();
        assert_eq!(menu.selected_option, 1);
        assert_eq!(menu.get_selected(), "Play vs AI");
    }

    #[test]
    fn test_menu_move_up_wraps() {
        let mut menu = Menu::new(MAIN_MENU_OPTIONS.to_vec());
        menu.move_up();
        assert_eq!(menu.selected_option, 2);
        assert_eq!(menu.get_selected(), "Quit");
    }

    #[test]
    fn test_menu_move_down_wraps() {
        let mut menu = Menu::new(TTT_MENU_OPTIONS.to_vec());
        menu.selected_option = 2;
        menu.move_down();
        assert_eq!(menu.selected_option, 0);
    }

    #[test]
    fn test_gameplay_new_pvp() {
        let game = GamePlayTTT::new(GameMode::LocalPvP);
        assert_eq!(game.mode, GameMode::LocalPvP);
        assert_eq!(game.active_player, Mark::X);
        assert_eq!(game.board.state, GameState::Playing);
        assert_eq!(game.turn, 0);
        assert!(game.ai.is_none());
    }

    #[test]
    fn test_gameplay_new_pve() {
        let game = GamePlayTTT::new(GameMode::PvE);
        assert_eq!(game.mode, GameMode::PvE);
        assert!(game.ai.is_some());
    }

    #[test]
    fn test_player_move_places_mark() {
        let mut game = GamePlayTTT::new(GameMode::LocalPvP);
        game.player_move();

        assert!(game.board.get(0, 0).is_some());
        assert_eq!(game.board.get(0, 0).unwrap(), Mark::X);
        assert_eq!(game.turn, 1);
    }

    #[test]
    fn test_player_move_switches_player() {
        let mut game = GamePlayTTT::new(GameMode::LocalPvP);
        assert_eq!(game.active_player, Mark::X);

        game.player_move();
        assert_eq!(game.active_player, Mark::O);

        game.input_right();
        game.player_move();
        assert_eq!(game.active_player, Mark::X);
    }

    #[test]
    fn test_player_move_detects_win() {
        let mut game = GamePlayTTT::new(GameMode::LocalPvP);
        // Set up winning position for X
        game.board.make_move(0, 0, Mark::X);
        game.board.make_move(0, 1, Mark::X);
        game.selected.row = 0;
        game.selected.col = 2;

        game.player_move();

        assert_eq!(game.board.state, GameState::Won(Mark::X));
    }

    #[test]
    fn test_reset_game() {
        let mut game = GamePlayTTT::new(GameMode::LocalPvP);
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
        let mut game = GamePlayTTT::new(GameMode::LocalPvP);
        game.selected.col = 2;

        game.input_right();
        assert_eq!(game.selected.col, 0);

        game.input_left();
        assert_eq!(game.selected.col, 2);
    }

    #[test]
    fn test_selection_wraps_vertically() {
        let mut game = GamePlayTTT::new(GameMode::LocalPvP);
        game.selected.row = 2;

        game.input_down();
        assert_eq!(game.selected.row, 0);

        game.input_up();
        assert_eq!(game.selected.row, 2);
    }
}
