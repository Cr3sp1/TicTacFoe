use crate::scenes::{GameMode, GamePlay, MainMenu};

pub enum CurrentScreen {
    MainMenu(MainMenu),
    Playing(GamePlay),
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_screen: CurrentScreen::MainMenu(MainMenu::new()),
            should_quit: false,
        }
    }

    pub fn start_game(&mut self, mode: GameMode) {
        self.current_screen = CurrentScreen::Playing(GamePlay::new(mode));
    }

    pub fn go_to_main_menu(&mut self) {
        self.current_screen = CurrentScreen::MainMenu(MainMenu::new());
    }

    pub fn handle_left(&mut self) {
        if let CurrentScreen::Playing(game) = &mut self.current_screen {
            game.input_left();
        }
    }

    pub fn handle_right(&mut self) {
        if let CurrentScreen::Playing(game) = &mut self.current_screen {
            game.input_right();
        }
    }

    pub fn handle_up(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::MainMenu(menu) => menu.move_up(),
            CurrentScreen::Playing(game) => game.input_up(),
        }
    }

    pub fn handle_down(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::MainMenu(menu) => menu.move_down(),
            CurrentScreen::Playing(game) => game.input_down(),
        }
    }

    pub fn handle_enter(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::MainMenu(menu) => match menu.get_selected() {
                "Local PvP" => self.start_game(GameMode::LocalPvP),
                "Play vs AI" => self.start_game(GameMode::PvE),
                "Quit" => self.should_quit = true,
                _ => panic!("Option selected in Main Menu does not exist."),
            },
            CurrentScreen::Playing(game) => game.make_move(),
        }
    }

    pub fn handle_second(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::Playing(game) => game.play_second(),
            _ => {}
        }
    }

    pub fn handle_reset(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::Playing(game) => game.reset_game(),
            _ => {}
        }
    }

    pub fn handle_main_menu(&mut self) {
        match &mut self.current_screen {
            CurrentScreen::Playing(_) => self.go_to_main_menu(),
            _ => {}
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
