use crate::scenes::{ConnectionMenu, GameMode, GamePlay, MainMenu};

/// Represents the current screen being displayed in the application.
pub enum CurrentScreen {
    MainMenu(MainMenu),
    ConnectionMenu(ConnectionMenu),
    Playing(GamePlay),
}


/// Main application state manager.
///
/// Handles screen transitions and delegates input events to the
/// appropriate screen handlers.
pub struct App {
    pub current_screen: CurrentScreen,
    pub should_quit: bool,
}

impl App {
    /// Creates a new App starting at the main menu.
    pub fn new() -> Self {
        Self {
            current_screen: CurrentScreen::MainMenu(MainMenu::new()),
            should_quit: false,
        }
    }

    /// Starts a new game with the specified mode.
    pub fn start_game(&mut self, mode: GameMode) {
        self.current_screen = CurrentScreen::Playing(GamePlay::new(mode));
    }

    pub fn go_to_connection_menu(&mut self) {
        self.current_screen = CurrentScreen::ConnectionMenu(ConnectionMenu{});
    }

    /// Returns to the main menu, discarding any active game.
    pub fn go_to_main_menu(&mut self) {
        self.current_screen = CurrentScreen::MainMenu(MainMenu::new());
    }

    /// Handles left arrow or 'h' key input.
    pub fn handle_left(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::Playing(game) => game.input_left(),
            _ => (),
        }
    }

    /// Handles right arrow or 'l' key input.
    pub fn handle_right(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::Playing(game) => game.input_right(),
            _ => (),
        }
    }

    /// Handles up arrow or 'k' key input.
    ///
    /// Moves menu selection up in main menu, or board selection up in game.
    pub fn handle_up(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::MainMenu(menu) => menu.move_up(),
            CurrentScreen::Playing(game) => game.input_up(),
            _ => (),
        }
    }

    /// Handles down arrow or 'j' key input.
    ///
    /// Moves menu selection down in main menu, or board selection down in game.
    pub fn handle_down(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::MainMenu(menu) => menu.move_down(),
            CurrentScreen::Playing(game) => game.input_down(),
            _ => (),
        }
    }

    /// Handles Enter or Space key input.
    ///
    /// Confirms menu selection or places a mark on the board.
    pub fn handle_enter(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::MainMenu(menu) => match menu.get_selected() {
                "Online PvP" => self.go_to_connection_menu(),
                "Local PvP" => self.start_game(GameMode::LocalPvP),
                "Play vs AI" => self.start_game(GameMode::PvE),
                "Quit" => self.should_quit = true,
                _ => panic!("Option selected in Main Menu does not exist."),
            },
            CurrentScreen::Playing(game) => game.make_move(),
            _ => (),
        }
    }

    /// Handles 's' key input to allow AI to play first in PvE mode.
    pub fn handle_second(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::Playing(game) => game.play_second(),
            _ => {}
        }
    }

    /// Handles 'r' key input to reset the current game.
    pub fn handle_reset(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::Playing(game) => game.reset_game(),
            _ => {}
        }
    }

    /// Handles 'm' key input to return to main menu from game.
    pub fn handle_main_menu(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::Playing(_) => self.go_to_main_menu(),
            CurrentScreen::ConnectionMenu(_) => self.go_to_main_menu(),
            _ => {}
        }
    }

    /// Sets the quit flag to exit the application.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new_starts_at_menu() {
        let app = App::new();
        assert!(matches!(app.current_screen, CurrentScreen::MainMenu(_)));
        assert!(!app.should_quit);
    }

    #[test]
    fn test_start_game_pvp() {
        let mut app = App::new();
        app.start_game(GameMode::LocalPvP);

        assert!(matches!(app.current_screen, CurrentScreen::Playing(_)));
    }

    #[test]
    fn test_start_game_pve() {
        let mut app = App::new();
        app.start_game(GameMode::PvE);

        if let CurrentScreen::Playing(game) = &app.current_screen {
            assert_eq!(game.mode, GameMode::PvE);
            assert!(game.ai.is_some());
        } else {
            panic!("Expected Playing screen");
        }
    }

    #[test]
    fn test_go_to_main_menu() {
        let mut app = App::new();
        app.start_game(GameMode::LocalPvP);
        app.go_to_main_menu();

        assert!(matches!(app.current_screen, CurrentScreen::MainMenu(_)));
    }

    #[test]
    fn test_quit_sets_flag() {
        let mut app = App::new();
        assert!(!app.should_quit);

        app.quit();
        assert!(app.should_quit);
    }

    #[test]
    fn test_handle_enter_in_menu_starts_connection_menu() {
        let mut app = App::new();
        // Default selection is "Online PvP"
        app.handle_enter();

        assert!(matches!(app.current_screen, CurrentScreen::ConnectionMenu(_)));
    }

    #[test]
    fn test_handle_reset_resets_game() {
        let mut app = App::new();
        app.start_game(GameMode::LocalPvP);

        // Make a move
        if let CurrentScreen::Playing(game) = &mut app.current_screen {
            game.make_move();
            assert!(game.turn > 0);
            assert!(game.board.get(0, 0).is_some());
        }

        app.handle_reset();

        if let CurrentScreen::Playing(game) = &app.current_screen {
            assert_eq!(game.turn, 0);
            assert!(game.board.get(0, 0).is_none());
        } else {
            panic!("Expected Playing screen");
        }
    }

    #[test]
    fn test_handle_main_menu_from_game() {
        let mut app = App::new();
        app.start_game(GameMode::LocalPvP);

        app.handle_main_menu();
        assert!(matches!(app.current_screen, CurrentScreen::MainMenu(_)));
    }

    #[test]
    fn test_handle_up_down_in_menu() {
        let mut app = App::new();

        let initial = match &app.current_screen {
            CurrentScreen::MainMenu(menu) => menu.selected_option,
            _ => panic!("Expected MainMenu"),
        };

        app.handle_up();

        match &app.current_screen {
            CurrentScreen::MainMenu(menu) => assert_ne!(menu.selected_option, initial),
            _ => panic!("Expected MainMenu"),
        }

        app.handle_down();

        match &app.current_screen {
            CurrentScreen::MainMenu(menu) => assert_eq!(menu.selected_option, initial),
            _ => panic!("Expected MainMenu"),
        }
    }
}
