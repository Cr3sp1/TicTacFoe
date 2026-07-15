use crate::ai::{AI, Game};
use crate::game::base::SmallBoard;
use crate::game::ultimate::BigBoard;
use crate::game::{Board, GameState, Mark};
use crate::utils::{
    Position, move_selection_down_playable, move_selection_left_playable,
    move_selection_right_playable, move_selection_up_playable, reset_position,
};

pub const MAIN_MENU_OPTIONS: [&'static str; 3] = ["Ultimate Tic Tac Toe", "Tic Tac Toe", "Quit"];
pub const TTT_MENU_OPTIONS: [&'static str; 5] =
    ["Play vs AI", "AI vs AI", "Local PvP", "Online PvP", "Back"];
pub const UTT_MENU_OPTIONS: [&'static str; 4] = ["Play vs AI", "AI vs AI", "Local PvP", "Back"];
pub const AI_MENU_OPTIONS: [&'static str; 4] = ["Weak", "Medium", "Strong", "Back"];
pub const ONLINE_TTT_MENU_OPTIONS: [&'static str; 3] = ["Host Match", "Join Match", "Back"];

/// Represents all the possible scenes.
pub enum Scene {
    MainMenu(Menu),
    TTTMenu(Menu),
    OnlineTTTMenu(Menu),
    HostingOnlineTTT,
    JoiningOnlineTTT(TicketInput),
    UTTMenu(Menu),
    AIMenu(Menu, AIMenuStatus),
    PlayingTTT(GamePlayTTT),
    PlayingUTT(GamePlayUTT),
}

/// Editable iroh ticket text used by the join screen.
#[derive(Default)]
pub struct TicketInput {
    pub value: String,
}

impl TicketInput {
    /// Appends typed or pasted ticket text.
    pub fn push_str(&mut self, value: &str) {
        self.value
            .extend(value.chars().filter(|character| !character.is_whitespace()));
    }

    /// Removes the final entered character.
    pub fn backspace(&mut self) {
        self.value.pop();
    }
}

pub enum AIMenuStatus {
    TTTpve,
    UTTpve,
    TTTeve(Option<AI>),
    UTTeve(Option<AI>),
}

/// Represents the game mode selection.
#[derive(Debug, Clone, PartialEq)]
pub enum GameMode {
    PvE(AI),
    EvE(AI, AI),
    LocalPvP,
    OnlinePvP(Mark),
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
    starting_player: Mark,
    local_rematch_ready: bool,
    remote_rematch_ready: bool,
}

impl GamePlayTTT {
    /// Creates a new game with the specified mode.
    pub fn new(mode: GameMode) -> Self {
        Self {
            board: SmallBoard::new(),
            active_player: Mark::X,
            turn: 0,
            mode,
            selected: Position { row: 0, col: 0 },
            starting_player: Mark::X,
            local_rematch_ready: false,
            remote_rematch_ready: false,
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
    pub fn play_move(&mut self) -> bool {
        if self.board.state != GameState::Playing {
            return false;
        }
        if matches!(
            self.mode,
            GameMode::OnlinePvP(local_mark) if local_mark != self.active_player
        ) {
            return false;
        }

        match self.mode {
            GameMode::EvE(_, _) => {}
            _ => {
                self.apply_move(self.selected.row, self.selected.col);
                if self.board.state != GameState::Playing {
                    return true;
                }
            }
        };

        self.ai_play();
        reset_position(&self.board, &mut self.selected);
        true
    }

    pub fn play_remote_move(&mut self, row: usize, col: usize) -> bool {
        let GameMode::OnlinePvP(local_mark) = self.mode else {
            return false;
        };
        if self.board.state != GameState::Playing
            || self.active_player == local_mark
            || row >= 3
            || col >= 3
            || !self.board.is_playable(row, col)
        {
            return false;
        }

        self.apply_move(row, col);
        reset_position(&self.board, &mut self.selected);
        true
    }

    pub fn request_online_rematch(&mut self) -> bool {
        if !matches!(self.mode, GameMode::OnlinePvP(_))
            || self.board.state == GameState::Playing
            || self.local_rematch_ready
        {
            return false;
        }

        self.local_rematch_ready = true;
        self.start_rematch_if_ready();
        true
    }

    pub fn receive_remote_rematch_ready(&mut self) -> bool {
        if !matches!(self.mode, GameMode::OnlinePvP(_)) || self.board.state == GameState::Playing {
            return false;
        }
        if self.remote_rematch_ready {
            return true;
        }

        self.remote_rematch_ready = true;
        self.start_rematch_if_ready();
        true
    }

    pub fn yield_online_first_move(&mut self) -> bool {
        let GameMode::OnlinePvP(local_mark) = self.mode else {
            return false;
        };
        if self.board.state != GameState::Playing
            || self.turn != 0
            || self.active_player != local_mark
        {
            return false;
        }

        self.start_online_round(local_mark.switch());
        true
    }

    pub fn apply_remote_yield_first_move(&mut self) -> bool {
        let GameMode::OnlinePvP(local_mark) = self.mode else {
            return false;
        };
        if self.board.state != GameState::Playing
            || self.turn != 0
            || self.active_player != local_mark.switch()
        {
            return false;
        }

        self.start_online_round(local_mark);
        true
    }

    pub fn waiting_for_rematch(&self) -> bool {
        self.local_rematch_ready
    }

    fn start_rematch_if_ready(&mut self) {
        if self.local_rematch_ready && self.remote_rematch_ready {
            self.start_online_round(self.starting_player.switch());
        }
    }

    fn start_online_round(&mut self, starting_player: Mark) {
        self.board = SmallBoard::new();
        self.active_player = starting_player;
        self.turn = 0;
        self.selected = Position { row: 0, col: 0 };
        self.starting_player = starting_player;
        self.local_rematch_ready = false;
        self.remote_rematch_ready = false;
    }

    fn apply_move(&mut self, row: usize, col: usize) {
        self.board.make_move(row, col, self.active_player);
        self.turn += 1;
        self.active_player = self.active_player.switch();
    }

    /// Executes the AI's turn in PvE and EvE modes.
    fn ai_play(&mut self) {
        match &mut self.mode {
            GameMode::LocalPvP | GameMode::OnlinePvP(_) => return,
            GameMode::PvE(ai) => {
                let (ai_row, ai_col) = ai.choose_move_ttt(&self.board).unwrap_base();
                self.board.make_move(ai_row, ai_col, ai.get_mark());

                self.turn += 1;
                self.active_player = self.active_player.switch();

                reset_position(&self.board, &mut self.selected);
            }
            GameMode::EvE(ai_x, ai_o) => {
                let (ai_row, ai_col) = match self.active_player {
                    Mark::X => ai_x.choose_move_ttt(&self.board).unwrap_base(),
                    Mark::O => ai_o.choose_move_ttt(&self.board).unwrap_base(),
                };
                self.board.make_move(ai_row, ai_col, self.active_player);

                self.turn += 1;
                self.active_player = self.active_player.switch()
            }
        }
    }

    /// Allows the O player to play first if the game just started.
    pub fn play_second(&mut self) {
        if matches!(self.mode, GameMode::OnlinePvP(_)) {
            return;
        }
        if self.board.state == GameState::Playing && self.turn == 0 {
            self.active_player = Mark::O;
            match &mut self.mode {
                GameMode::LocalPvP | GameMode::OnlinePvP(_) => return,
                GameMode::PvE(ai) => ai.switch_starting_mark(),
                GameMode::EvE(ai_x, ai_o) => {
                    ai_x.switch_starting_mark();
                    ai_o.switch_starting_mark();
                }
            }
            self.ai_play();
        }
    }

    /// Resets the game to initial state while keeping the same mode.
    pub fn reset_game(&mut self) {
        if matches!(self.mode, GameMode::OnlinePvP(_)) {
            return;
        }
        self.board = SmallBoard::new();
        self.active_player = Mark::X;
        self.turn = 0;
        self.selected.row = 0;
        self.selected.col = 0;
        match &mut self.mode {
            GameMode::LocalPvP | GameMode::OnlinePvP(_) => return,
            GameMode::PvE(ai) => ai.reset(),
            GameMode::EvE(ai_x, ai_o) => {
                ai_x.reset();
                ai_o.reset();
            }
        }
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
        if self.selected_cell.is_none() && !matches!(self.mode, GameMode::EvE(_, _)) {
            let mut cell_position = Position { row: 0, col: 0 };
            let selected_board = self
                .big_board
                .get_board(self.selected_board.row, self.selected_board.col);
            reset_position(selected_board, &mut cell_position);
            self.selected_cell = Some(cell_position);
        } else {
            self.play_move();
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
    pub fn play_move(&mut self) {
        if self.big_board.state != GameState::Playing {
            return;
        }

        match self.mode {
            GameMode::EvE(_, _) => {}
            GameMode::OnlinePvP(_) => return,
            _ => {
                let selected_cell = self.selected_cell.as_mut().unwrap();
                self.big_board.make_move(
                    self.selected_board.row,
                    self.selected_board.col,
                    selected_cell.row,
                    selected_cell.col,
                    self.active_player,
                );

                self.turn += 1;

                self.active_player = self.active_player.switch();

                if self.big_board.state != GameState::Playing {
                    return;
                }
            }
        };

        self.ai_play();

        self.reset_selection()
    }

    fn reset_selection(&mut self) {
        // If next action is constrained to active board, select it by default
        if let Some((active_row, active_col)) = self.big_board.active_board {
            self.selected_board = Position {
                row: active_row,
                col: active_col,
            };
            self.selected_cell = Some(Position { row: 0, col: 0 });
            reset_position(
                self.big_board.get_board(active_row, active_col),
                self.selected_cell.as_mut().unwrap(),
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
        match &mut self.mode {
            GameMode::LocalPvP | GameMode::OnlinePvP(_) => return,
            GameMode::PvE(ai) => ai.reset(),
            GameMode::EvE(ai_x, ai_o) => {
                ai_x.reset();
                ai_o.reset();
            }
        }
    }

    /// Executes the AI's turn in PvE and EvE modes.
    fn ai_play(&mut self) {
        match &mut self.mode {
            GameMode::LocalPvP | GameMode::OnlinePvP(_) => return,
            GameMode::PvE(ai) => {
                let mv = ai.choose_move_utt(&self.big_board);
                self.big_board.play(&mv, ai.get_mark());

                self.turn += 1;
                self.active_player = self.active_player.switch();

                self.reset_selection();
            }
            GameMode::EvE(ai_x, ai_o) => {
                let mv = match self.active_player {
                    Mark::X => ai_x.choose_move_utt(&self.big_board),
                    Mark::O => ai_o.choose_move_utt(&self.big_board),
                };
                self.big_board.play(&mv, self.active_player);

                self.turn += 1;
                self.active_player = self.active_player.switch();
            }
        }
    }

    /// Allows the AI to play first if the game just started.
    pub fn play_second(&mut self) {
        if self.big_board.state == GameState::Playing && self.turn == 0 {
            self.active_player = Mark::O;
            match &mut self.mode {
                GameMode::LocalPvP | GameMode::OnlinePvP(_) => return,
                GameMode::PvE(ai) => ai.switch_starting_mark(),
                GameMode::EvE(ai_x, ai_o) => {
                    ai_x.switch_starting_mark();
                    ai_o.switch_starting_mark();
                }
            }
            self.ai_play();
        }
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
        assert_eq!(menu.options.len(), TTT_MENU_OPTIONS.len());
        assert_eq!(menu.get_selected(), "Play vs AI");
    }

    #[test]
    fn test_menu_move_down() {
        let mut menu = Menu::new(TTT_MENU_OPTIONS.to_vec());
        menu.move_down();
        assert_eq!(menu.selected_option, 1);
        assert_eq!(menu.get_selected(), "AI vs AI");
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
        menu.selected_option = menu.options.len() - 1;
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
    }

    #[test]
    fn test_gameplay_new_pve() {
        let game = GamePlayTTT::new(GameMode::PvE(AI::Weak(Mark::X)));
        assert_eq!(game.mode, GameMode::PvE(AI::Weak(Mark::X)));
    }

    #[test]
    fn test_player_move_places_mark() {
        let mut game = GamePlayTTT::new(GameMode::LocalPvP);
        game.play_move();

        assert!(game.board.get(0, 0).is_some());
        assert_eq!(game.board.get(0, 0).unwrap(), Mark::X);
        assert_eq!(game.turn, 1);
    }

    #[test]
    fn test_player_move_switches_player() {
        let mut game = GamePlayTTT::new(GameMode::LocalPvP);
        assert_eq!(game.active_player, Mark::X);

        game.play_move();
        assert_eq!(game.active_player, Mark::O);

        game.input_right();
        game.play_move();
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

        game.play_move();

        assert_eq!(game.board.state, GameState::Won(Mark::X));
    }

    #[test]
    fn test_online_player_can_move_only_on_their_turn() {
        let mut game = GamePlayTTT::new(GameMode::OnlinePvP(Mark::O));

        assert!(!game.play_move());
        assert_eq!(game.turn, 0);
        assert!(game.board.get(0, 0).is_none());

        assert!(game.play_remote_move(0, 0));
        assert_eq!(game.board.get(0, 0), Some(Mark::X));
        assert_eq!(game.active_player, Mark::O);

        game.selected = Position { row: 1, col: 1 };
        assert!(game.play_move());
        assert_eq!(game.board.get(1, 1), Some(Mark::O));
    }

    #[test]
    fn test_online_remote_move_must_be_valid_and_on_opponents_turn() {
        let mut game = GamePlayTTT::new(GameMode::OnlinePvP(Mark::O));

        assert!(game.play_remote_move(0, 0));
        assert!(!game.play_remote_move(0, 1));

        game.selected = Position { row: 1, col: 1 };
        assert!(game.play_move());
        assert!(!game.play_remote_move(1, 1));
        assert!(!game.play_remote_move(3, 0));
        assert_eq!(game.turn, 2);
    }

    #[test]
    fn test_online_rematch_waits_for_both_players() {
        let mut game = GamePlayTTT::new(GameMode::OnlinePvP(Mark::O));
        game.board.set(0, 0, Some(Mark::X));
        game.board.state = GameState::Won(Mark::X);
        game.turn = 5;

        assert!(game.request_online_rematch());
        assert!(game.waiting_for_rematch());
        assert_eq!(game.board.state, GameState::Won(Mark::X));
        assert!(!game.request_online_rematch());

        assert!(game.receive_remote_rematch_ready());
        assert_eq!(game.board.state, GameState::Playing);
        assert_eq!(game.active_player, Mark::O);
        assert_eq!(game.turn, 0);
        assert!(game.board.get(0, 0).is_none());
    }

    #[test]
    fn test_rematch_starts_with_previous_second_player() {
        let mut game = GamePlayTTT::new(GameMode::OnlinePvP(Mark::X));
        game.board.state = GameState::Draw;

        assert!(game.receive_remote_rematch_ready());
        assert_eq!(game.board.state, GameState::Draw);
        assert!(game.request_online_rematch());

        assert_eq!(game.board.state, GameState::Playing);
        assert_eq!(game.active_player, Mark::O);
    }

    #[test]
    fn test_yield_updates_next_rematch_order() {
        let mut game = GamePlayTTT::new(GameMode::OnlinePvP(Mark::X));

        assert!(game.yield_online_first_move());
        assert_eq!(game.active_player, Mark::O);
        assert!(!game.yield_online_first_move());

        game.board.state = GameState::Draw;
        assert!(game.request_online_rematch());
        assert!(game.receive_remote_rematch_ready());
        assert_eq!(game.active_player, Mark::X);
    }

    #[test]
    fn test_remote_yield_requires_remote_to_own_first_move() {
        let mut game = GamePlayTTT::new(GameMode::OnlinePvP(Mark::O));

        assert!(game.apply_remote_yield_first_move());
        assert_eq!(game.active_player, Mark::O);
        assert!(!game.apply_remote_yield_first_move());
    }

    #[test]
    fn test_online_game_ignores_local_starting_player_and_reset_controls() {
        let mut game = GamePlayTTT::new(GameMode::OnlinePvP(Mark::X));
        assert!(game.play_move());

        game.play_second();
        game.reset_game();

        assert_eq!(game.turn, 1);
        assert_eq!(game.active_player, Mark::O);
        assert_eq!(game.board.get(0, 0), Some(Mark::X));
    }

    #[test]
    fn test_reset_game() {
        let mut game = GamePlayTTT::new(GameMode::LocalPvP);
        game.play_move();
        game.input_right();
        game.play_move();

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
