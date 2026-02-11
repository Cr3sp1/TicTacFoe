use crate::ai::AI::{Medium, Weak};
use crate::ai::simple::SimpleAi;
use crate::game::Mark::{O};
use crate::scenes::{
    AI_MENU_OPTIONS, AIMenuStatus, GameMode, GamePlayTTT, GamePlayUTT, MAIN_MENU_OPTIONS, Menu,
    Scene, TTT_MENU_OPTIONS, UTT_MENU_OPTIONS,
};

/// Main application state manager.
///
/// Handles screen transitions and delegates input events to the
/// appropriate screen handlers.
pub struct App {
    pub current_scene: Scene,
    pub should_quit: bool,
}

impl App {
    /// Creates a new App starting at the main menu.
    pub fn new() -> Self {
        Self {
            current_scene: Scene::MainMenu(Menu::new(vec![
                "Ultimate Tic Tac Toe",
                "Tic Tac Toe",
                "Quit",
            ])),
            should_quit: false,
        }
    }

    /// Starts a new tic-tac-toe game with the specified mode.
    pub fn start_ttt_game(&mut self, mode: GameMode) {
        self.current_scene = Scene::PlayingTTT(GamePlayTTT::new(mode));
    }

    /// Starts a new ultimate tic-tac-toe game with the specified mode.
    pub fn start_utt_game(&mut self, mode: GameMode) {
        self.current_scene = Scene::PlayingUTT(GamePlayUTT::new(mode));
    }

    /// Goes to the main menu, discarding any active game.
    pub fn go_to_main_menu(&mut self) {
        self.current_scene = Scene::MainMenu(Menu::new(MAIN_MENU_OPTIONS.to_vec()));
    }

    /// Goes to the tic-tac-toe menu.
    pub fn go_to_ttt_menu(&mut self) {
        self.current_scene = Scene::TTTMenu(Menu::new(TTT_MENU_OPTIONS.to_vec()));
    }

    /// Goes to the ultimate tic-tac-toe menu.
    pub fn go_to_utt_menu(&mut self) {
        self.current_scene = Scene::UTTMenu(Menu::new(UTT_MENU_OPTIONS.to_vec()));
    }

    /// Goes to the AI menu.
    pub fn go_to_ai_menu(&mut self, status: AIMenuStatus) {
        self.current_scene = Scene::AIMenu(Menu::new(AI_MENU_OPTIONS.to_vec()), status);
    }

    /// Handles left arrow or 'h' key input.
    pub fn handle_left(&mut self) {
        match &mut self.current_scene {
            Scene::PlayingTTT(game) => game.input_left(),
            Scene::PlayingUTT(game) => game.input_left(),
            _ => {}
        }
    }

    /// Handles right arrow or 'l' key input.
    pub fn handle_right(&mut self) {
        match &mut self.current_scene {
            Scene::PlayingTTT(game) => game.input_right(),
            Scene::PlayingUTT(game) => game.input_right(),
            _ => {}
        }
    }

    /// Handles up arrow or 'k' key input.
    ///
    /// Moves menu selection up in main menu, or board selection up in game.
    pub fn handle_up(&mut self) {
        match &mut self.current_scene {
            Scene::MainMenu(menu)
            | Scene::TTTMenu(menu)
            | Scene::UTTMenu(menu)
            | Scene::AIMenu(menu, _) => menu.move_up(),
            Scene::PlayingTTT(game) => game.input_up(),
            Scene::PlayingUTT(game) => game.input_up(),
        }
    }

    /// Handles down arrow or 'j' key input.
    ///
    /// Moves menu selection down in main menu, or board selection down in game.
    pub fn handle_down(&mut self) {
        match &mut self.current_scene {
            Scene::MainMenu(menu)
            | Scene::TTTMenu(menu)
            | Scene::UTTMenu(menu)
            | Scene::AIMenu(menu, _) => menu.move_down(),
            Scene::PlayingTTT(game) => game.input_down(),
            Scene::PlayingUTT(game) => game.input_down(),
        }
    }

    /// Handles Enter or Space key input.
    ///
    /// Confirms menu selection or places a mark on the board.
    pub fn handle_enter(&mut self) {
        match &mut self.current_scene {
            Scene::MainMenu(menu) => match menu.get_selected() {
                "Ultimate Tic Tac Toe" => self.go_to_utt_menu(),
                "Tic Tac Toe" => self.go_to_ttt_menu(),
                "Quit" => self.should_quit = true,
                _ => panic!("Option selected in Tic Tac Toe Menu does not exist."),
            },
            Scene::TTTMenu(menu) => match menu.get_selected() {
                "Local PvP" => self.start_ttt_game(GameMode::LocalPvP),
                "Play vs AI" => self.go_to_ai_menu(AIMenuStatus::TTTpve),
                "Back" => self.go_to_main_menu(),
                _ => panic!("Option selected in Tic Tac Toe Menu does not exist."),
            },
            Scene::UTTMenu(menu) => match menu.get_selected() {
                "Local PvP" => self.start_utt_game(GameMode::LocalPvP),
                "Play vs AI" => self.go_to_ai_menu(AIMenuStatus::UTTpve),
                "Back" => self.go_to_main_menu(),
                _ => panic!("Option selected in Ultimate Tic Tac Toe Menu does not exist."),
            },
            Scene::AIMenu(menu, status) => match (menu.get_selected(), status) {
                ("Weak", AIMenuStatus::TTTpve) => self.start_ttt_game(GameMode::PvE(Weak(O))),
                ("Weak", AIMenuStatus::UTTpve) => self.start_utt_game(GameMode::PvE(Weak(O))),
                ("Medium", AIMenuStatus::TTTpve) => {
                    self.start_ttt_game(GameMode::PvE(Medium(SimpleAi::new(O))))
                }
                ("Medium", AIMenuStatus::UTTpve) => {
                    self.start_ttt_game(GameMode::PvE(Medium(SimpleAi::new(O))))
                }
                (_, _) => panic!("Option selected in AI Menu does not exist."),
            },
            Scene::PlayingTTT(game) => game.player_move(),
            Scene::PlayingUTT(game) => game.input_enter(),
        }
    }

    /// Handles Esc key input.
    ///
    /// Goes back to previous menu/move selection.
    pub fn handle_esc(&mut self) {
        match &mut self.current_scene {
            Scene::MainMenu(_) => self.quit(),
            Scene::TTTMenu(_) => self.go_to_main_menu(),
            Scene::UTTMenu(_) => self.go_to_main_menu(),
            Scene::AIMenu(_, status) => match status {
                AIMenuStatus::TTTpve => self.go_to_ttt_menu(),
                AIMenuStatus::UTTpve => self.go_to_utt_menu(),
            },
            Scene::PlayingUTT(game) => game.input_esc(),
            Scene::PlayingTTT(_) => {}
        }
    }

    /// Handles 's' key input to allow AI to play first in PvE mode.
    pub fn handle_second(&mut self) {
        match &mut self.current_scene {
            Scene::PlayingTTT(game) => game.play_second(),
            Scene::PlayingUTT(game) => game.play_second(),
            _ => {}
        }
    }

    /// Handles 'r' key input to reset the current game.
    pub fn handle_reset(&mut self) {
        match &mut self.current_scene {
            Scene::PlayingTTT(game) => game.reset_game(),
            Scene::PlayingUTT(game) => game.reset_game(),
            _ => {}
        }
    }

    /// Handles 'm' key input to return to main menu from game.
    pub fn handle_main_menu(&mut self) {
        match &mut self.current_scene {
            Scene::PlayingTTT(_) | Scene::PlayingUTT(_) => self.go_to_main_menu(),
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
    use crate::game::{Mark::X, Board};

    #[test]
    fn test_app_new_starts_at_menu() {
        let app = App::new();
        assert!(matches!(app.current_scene, Scene::MainMenu(_)));
        assert!(!app.should_quit);
    }

    #[test]
    fn test_start_game_pvp() {
        let mut app = App::new();
        app.start_ttt_game(GameMode::LocalPvP);

        assert!(matches!(app.current_scene, Scene::PlayingTTT(_)));
    }

    #[test]
    fn test_start_game_pve() {
        let mut app = App::new();
        app.start_ttt_game(GameMode::PvE(Weak(X)));

        if let Scene::PlayingTTT(game) = &app.current_scene {
            assert_eq!(game.mode, GameMode::PvE(Weak(X)));
        } else {
            panic!("Expected Playing screen");
        }
    }

    #[test]
    fn test_go_to_main_menu() {
        let mut app = App::new();
        app.start_ttt_game(GameMode::LocalPvP);
        app.go_to_main_menu();

        assert!(matches!(app.current_scene, Scene::MainMenu(_)));
    }

    #[test]
    fn test_quit_sets_flag() {
        let mut app = App::new();
        assert!(!app.should_quit);

        app.quit();
        assert!(app.should_quit);
    }

    #[test]
    fn test_start_game_from_menus() {
        let mut app = App::new();
        app.handle_down();
        app.handle_enter();
        app.handle_enter();

        assert!(matches!(app.current_scene, Scene::PlayingTTT(_)));
    }

    #[test]
    fn test_handle_reset_resets_game() {
        let mut app = App::new();
        app.start_ttt_game(GameMode::LocalPvP);

        // Make a move
        if let Scene::PlayingTTT(game) = &mut app.current_scene {
            game.player_move();
            assert!(game.turn > 0);
            assert!(game.board.get(0, 0).is_some());
        }

        app.handle_reset();

        if let Scene::PlayingTTT(game) = &app.current_scene {
            assert_eq!(game.turn, 0);
            assert!(game.board.get(0, 0).is_none());
        } else {
            panic!("Expected Playing screen");
        }
    }

    #[test]
    fn test_handle_main_menu_from_game() {
        let mut app = App::new();
        app.start_ttt_game(GameMode::LocalPvP);

        app.handle_main_menu();
        assert!(matches!(app.current_scene, Scene::MainMenu(_)));
    }

    #[test]
    fn test_handle_up_down_in_menu() {
        let mut app = App::new();

        let initial = match &app.current_scene {
            Scene::MainMenu(menu) => menu.selected_option,
            _ => panic!("Expected MainMenu"),
        };

        app.handle_up();

        match &app.current_scene {
            Scene::MainMenu(menu) => assert_ne!(menu.selected_option, initial),
            _ => panic!("Expected MainMenu"),
        }

        app.handle_down();

        match &app.current_scene {
            Scene::MainMenu(menu) => assert_eq!(menu.selected_option, initial),
            _ => panic!("Expected MainMenu"),
        }
    }
}
