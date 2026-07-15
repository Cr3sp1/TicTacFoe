use crate::ai::AI;
use crate::ai::AI::{Medium, StrongTTT, StrongUTT, Weak};
use crate::ai::mcts::MCTSAi;
use crate::ai::simple::SimpleAi;
use crate::game::Mark;
use crate::game::Mark::{O, X};
use crate::game::base::SmallBoard;
use crate::game::ultimate::BigBoard;
use crate::network::protocol::{GameVariant, MoveMessage};
use crate::network::{NetworkClient, NetworkCommand, NetworkEvent, NetworkStatus};
use crate::scenes::{
    AI_MENU_OPTIONS, AIMenuStatus, GameMode, GamePlayTTT, GamePlayUTT, MAIN_MENU_OPTIONS, Menu,
    ONLINE_TTT_MENU_OPTIONS, Scene, TTT_MENU_OPTIONS, TicketInput, UTT_MENU_OPTIONS,
};

/// Main application state manager.
///
/// Handles screen transitions and delegates input events to the
/// appropriate screen handlers.
pub struct App {
    pub current_scene: Scene,
    pub network_status: NetworkStatus,
    pub should_quit: bool,
    network_client: Option<NetworkClient>,
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
            network_status: NetworkStatus::Idle,
            should_quit: false,
            network_client: None,
        }
    }

    /// Starts the network worker if it is not already running.
    pub fn start_network(&mut self) -> std::io::Result<()> {
        if self.network_client.is_none() {
            self.network_client = Some(NetworkClient::start()?);
        }
        Ok(())
    }

    /// Stops the network worker and resets its visible status.
    pub fn stop_network(&mut self) {
        self.network_client = None;
        self.network_status = NetworkStatus::Idle;
    }

    /// Returns whether the network worker is currently running.
    pub fn network_is_active(&self) -> bool {
        self.network_client.is_some()
    }

    /// Starts hosting an online match.
    pub fn host_online_match(&mut self, game: GameVariant) -> std::io::Result<()> {
        self.send_network_command(NetworkCommand::Host(game))
    }

    /// Attempts to join an online match using an iroh ticket.
    pub fn join_online_match(&mut self, ticket: String, game: GameVariant) -> std::io::Result<()> {
        self.send_network_command(NetworkCommand::Join { ticket, game })
    }

    /// Disconnects the current online match without stopping the worker.
    pub fn disconnect_online_match(&mut self) -> std::io::Result<()> {
        if self.network_client.is_none() {
            return Ok(());
        }
        self.send_active_network_command(NetworkCommand::Disconnect)
    }

    /// Applies all network events currently waiting for the synchronous app loop.
    pub fn poll_network_events(&mut self) {
        loop {
            let Some(client) = &self.network_client else {
                return;
            };
            let Ok(event) = client.try_recv() else {
                return;
            };
            self.handle_network_event(event);
        }
    }

    fn handle_network_event(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::Connected { mark, game } => {
                self.network_status = NetworkStatus::Connected { mark };
                match game {
                    GameVariant::Classic => self.start_ttt_game(GameMode::OnlinePvP(mark)),
                    GameVariant::Ultimate => self.start_utt_game(GameMode::OnlinePvP(mark)),
                }
            }
            NetworkEvent::MoveReceived(message) => {
                let applied = match &mut self.current_scene {
                    Scene::PlayingTTT(game) => game.play_remote_move(message.row(), message.col()),
                    _ => false,
                };
                if !applied {
                    self.network_status =
                        NetworkStatus::Failed("received an invalid online move".to_string());
                }
            }
            NetworkEvent::RematchReadyReceived => {
                let applied = match &mut self.current_scene {
                    Scene::PlayingTTT(game) => game.receive_remote_rematch_ready(),
                    _ => false,
                };
                if !applied {
                    self.network_status =
                        NetworkStatus::Failed("received invalid rematch readiness".to_string());
                }
            }
            NetworkEvent::YieldFirstMoveReceived => {
                let applied = match &mut self.current_scene {
                    Scene::PlayingTTT(game) => game.apply_remote_yield_first_move(),
                    _ => false,
                };
                if !applied {
                    self.network_status =
                        NetworkStatus::Failed("received an invalid first-move yield".to_string());
                }
            }
            NetworkEvent::OpponentConceded => {
                let applied = match &mut self.current_scene {
                    Scene::PlayingTTT(game) => game.apply_remote_concession(),
                    _ => false,
                };
                if !applied {
                    self.network_status =
                        NetworkStatus::Failed("received an invalid concession".to_string());
                }
            }
            event => {
                if let Some(status) = event.into_status() {
                    self.network_status = status;
                }
            }
        }
    }

    fn send_online_action(&mut self, command: NetworkCommand) {
        if let Err(error) = self.send_active_network_command(command) {
            self.network_status = NetworkStatus::Failed(error.to_string());
        }
    }

    fn send_network_command(&mut self, command: NetworkCommand) -> std::io::Result<()> {
        self.start_network()?;
        self.send_active_network_command(command)
    }

    fn send_active_network_command(&mut self, command: NetworkCommand) -> std::io::Result<()> {
        let Some(client) = self.network_client.as_ref() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "network worker is not running",
            ));
        };
        let result = client.send(command);

        if result.is_err() {
            self.stop_network();
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "network worker stopped",
            ));
        }

        Ok(())
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

    /// Goes to the online tic-tac-toe menu.
    pub fn go_to_online_ttt_menu(&mut self) {
        self.current_scene = Scene::OnlineTTTMenu(Menu::new(ONLINE_TTT_MENU_OPTIONS.to_vec()));
    }

    /// Starts hosting a classic online match.
    pub fn start_hosting_online_ttt(&mut self) {
        self.current_scene = Scene::HostingOnlineTTT;
        self.network_status = NetworkStatus::Idle;
        if let Err(error) = self.host_online_match(GameVariant::Classic) {
            self.network_status = NetworkStatus::Failed(error.to_string());
        }
    }

    /// Opens the ticket-entry screen for joining a classic online match.
    pub fn start_joining_online_ttt(&mut self) {
        self.network_status = NetworkStatus::Idle;
        self.current_scene = Scene::JoiningOnlineTTT(TicketInput::default());
    }

    /// Adds pasted or typed text to the active ticket input.
    pub fn handle_text_input(&mut self, value: &str) -> bool {
        if !matches!(
            &self.network_status,
            NetworkStatus::Idle | NetworkStatus::Failed(_)
        ) {
            return false;
        }
        let Scene::JoiningOnlineTTT(input) = &mut self.current_scene else {
            return false;
        };
        input.push_str(value);
        if matches!(&self.network_status, NetworkStatus::Failed(_)) {
            self.network_status = NetworkStatus::Idle;
        }
        true
    }

    /// Removes the final character from the active ticket input.
    pub fn handle_backspace(&mut self) {
        if matches!(
            &self.network_status,
            NetworkStatus::Idle | NetworkStatus::Failed(_)
        ) {
            if let Scene::JoiningOnlineTTT(input) = &mut self.current_scene {
                input.backspace();
                if matches!(&self.network_status, NetworkStatus::Failed(_)) {
                    self.network_status = NetworkStatus::Idle;
                }
            }
        }
    }

    fn submit_joining_online_ttt(&mut self) {
        if !matches!(
            &self.network_status,
            NetworkStatus::Idle | NetworkStatus::Failed(_)
        ) {
            return;
        }
        let Scene::JoiningOnlineTTT(input) = &self.current_scene else {
            return;
        };
        if input.value.trim().is_empty() {
            self.network_status = NetworkStatus::Failed("ticket cannot be empty".to_string());
            return;
        }
        let ticket = input.value.clone();
        self.network_status = NetworkStatus::Connecting;
        if let Err(error) = self.join_online_match(ticket, GameVariant::Classic) {
            self.network_status = NetworkStatus::Failed(error.to_string());
        }
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
        let online_frozen = matches!(self.network_status, NetworkStatus::OpponentDisconnected);
        match &mut self.current_scene {
            Scene::PlayingTTT(game)
                if !online_frozen || !matches!(game.mode, GameMode::OnlinePvP(_)) =>
            {
                game.input_left()
            }
            Scene::PlayingUTT(game) => game.input_left(),
            _ => {}
        }
    }

    /// Handles right arrow or 'l' key input.
    pub fn handle_right(&mut self) {
        let online_frozen = matches!(self.network_status, NetworkStatus::OpponentDisconnected);
        match &mut self.current_scene {
            Scene::PlayingTTT(game)
                if !online_frozen || !matches!(game.mode, GameMode::OnlinePvP(_)) =>
            {
                game.input_right()
            }
            Scene::PlayingUTT(game) => game.input_right(),
            _ => {}
        }
    }

    /// Handles up arrow or 'k' key input.
    ///
    /// Moves menu selection up in main menu, or board selection up in game.
    pub fn handle_up(&mut self) {
        let online_frozen = matches!(self.network_status, NetworkStatus::OpponentDisconnected);
        match &mut self.current_scene {
            Scene::MainMenu(menu)
            | Scene::TTTMenu(menu)
            | Scene::OnlineTTTMenu(menu)
            | Scene::UTTMenu(menu)
            | Scene::AIMenu(menu, _) => menu.move_up(),
            Scene::HostingOnlineTTT | Scene::JoiningOnlineTTT(_) => {}
            Scene::PlayingTTT(game)
                if !online_frozen || !matches!(game.mode, GameMode::OnlinePvP(_)) =>
            {
                game.input_up()
            }
            Scene::PlayingTTT(_) => {}
            Scene::PlayingUTT(game) => game.input_up(),
        }
    }

    /// Handles down arrow or 'j' key input.
    ///
    /// Moves menu selection down in main menu, or board selection down in game.
    pub fn handle_down(&mut self) {
        let online_frozen = matches!(self.network_status, NetworkStatus::OpponentDisconnected);
        match &mut self.current_scene {
            Scene::MainMenu(menu)
            | Scene::TTTMenu(menu)
            | Scene::OnlineTTTMenu(menu)
            | Scene::UTTMenu(menu)
            | Scene::AIMenu(menu, _) => menu.move_down(),
            Scene::HostingOnlineTTT | Scene::JoiningOnlineTTT(_) => {}
            Scene::PlayingTTT(game)
                if !online_frozen || !matches!(game.mode, GameMode::OnlinePvP(_)) =>
            {
                game.input_down()
            }
            Scene::PlayingTTT(_) => {}
            Scene::PlayingUTT(game) => game.input_down(),
        }
    }

    fn play_ttt_move(&mut self) {
        let online_connected = matches!(self.network_status, NetworkStatus::Connected { .. });
        let message = {
            let Scene::PlayingTTT(game) = &mut self.current_scene else {
                return;
            };
            let selected = game.selected;
            let is_online = matches!(game.mode, GameMode::OnlinePvP(_));
            if is_online && !online_connected {
                return;
            }

            if game.play_move() && is_online {
                let row = u8::try_from(selected.row).expect("board row fits in u8");
                let col = u8::try_from(selected.col).expect("board column fits in u8");
                Some(MoveMessage::new(row, col).expect("selected board position is valid"))
            } else {
                None
            }
        };

        if let Some(message) = message
            && let Err(error) = self.send_active_network_command(NetworkCommand::SendMove(message))
        {
            self.network_status = NetworkStatus::Failed(error.to_string());
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
                "AI vs AI" => self.go_to_ai_menu(AIMenuStatus::TTTeve(None)),
                "Online PvP" => self.go_to_online_ttt_menu(),
                "Back" => self.go_to_main_menu(),
                _ => panic!("Option selected in Tic Tac Toe Menu does not exist."),
            },
            Scene::UTTMenu(menu) => match menu.get_selected() {
                "Local PvP" => self.start_utt_game(GameMode::LocalPvP),
                "Play vs AI" => self.go_to_ai_menu(AIMenuStatus::UTTpve),
                "AI vs AI" => self.go_to_ai_menu(AIMenuStatus::UTTeve(None)),
                "Back" => self.go_to_main_menu(),
                _ => panic!("Option selected in Ultimate Tic Tac Toe Menu does not exist."),
            },
            Scene::OnlineTTTMenu(menu) => match menu.get_selected() {
                "Host Match" => self.start_hosting_online_ttt(),
                "Join Match" => self.start_joining_online_ttt(),
                "Back" => self.go_to_ttt_menu(),
                _ => panic!("Option selected in Online Tic Tac Toe Menu does not exist."),
            },
            Scene::AIMenu(menu, status) => {
                let selected_option = menu.get_selected();
                if selected_option == "Back" {
                    match &status {
                        AIMenuStatus::TTTpve => self.go_to_ttt_menu(),
                        AIMenuStatus::TTTeve(None) => self.go_to_ttt_menu(),
                        AIMenuStatus::TTTeve(Some(_)) => {
                            self.go_to_ai_menu(AIMenuStatus::TTTeve(None))
                        }
                        AIMenuStatus::UTTpve => self.go_to_utt_menu(),
                        AIMenuStatus::UTTeve(None) => self.go_to_utt_menu(),
                        AIMenuStatus::UTTeve(Some(_)) => {
                            self.go_to_ai_menu(AIMenuStatus::UTTeve(None))
                        }
                    }
                    return;
                }
                let new_ai = match selected_option {
                    "Weak" => |mark: Mark| -> AI { Weak(mark) },
                    "Medium" => |mark: Mark| -> AI { Medium(SimpleAi::new(mark)) },
                    "Strong" => match &status {
                        AIMenuStatus::TTTpve | AIMenuStatus::TTTeve(_) => {
                            |mark| -> AI { StrongTTT(MCTSAi::new(SmallBoard::new(), mark)) }
                        }
                        AIMenuStatus::UTTpve | AIMenuStatus::UTTeve(_) => {
                            |mark| -> AI { StrongUTT(MCTSAi::new(BigBoard::new(), mark)) }
                        }
                    },
                    _ => panic!("Option selected in AI Menu does not exist."),
                };
                match status {
                    AIMenuStatus::TTTpve => self.start_ttt_game(GameMode::PvE(new_ai(O))),
                    AIMenuStatus::UTTpve => self.start_utt_game(GameMode::PvE(new_ai(O))),
                    AIMenuStatus::TTTeve(None) => {
                        self.go_to_ai_menu(AIMenuStatus::TTTeve(Some(new_ai(X))))
                    }
                    AIMenuStatus::TTTeve(Some(ai_x)) => {
                        let new_mode = GameMode::EvE(ai_x.clone(), new_ai(O));
                        self.start_ttt_game(new_mode);
                    }
                    AIMenuStatus::UTTeve(None) => {
                        self.go_to_ai_menu(AIMenuStatus::UTTeve(Some(new_ai(X))))
                    }
                    AIMenuStatus::UTTeve(Some(ai_x)) => {
                        let new_mode = GameMode::EvE(ai_x.clone(), new_ai(O));
                        self.start_utt_game(new_mode);
                    }
                }
            }
            Scene::HostingOnlineTTT => {}
            Scene::JoiningOnlineTTT(_) => self.submit_joining_online_ttt(),
            Scene::PlayingTTT(_) => self.play_ttt_move(),
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
            Scene::OnlineTTTMenu(_) => self.go_to_ttt_menu(),
            Scene::HostingOnlineTTT => {
                self.stop_network();
                self.go_to_online_ttt_menu();
            }
            Scene::JoiningOnlineTTT(_) => {
                self.stop_network();
                self.go_to_online_ttt_menu();
            }
            Scene::UTTMenu(_) => self.go_to_main_menu(),
            Scene::AIMenu(_, status) => match status {
                AIMenuStatus::TTTpve => self.go_to_ttt_menu(),
                AIMenuStatus::TTTeve(None) => self.go_to_ttt_menu(),
                AIMenuStatus::TTTeve(Some(_)) => self.go_to_ai_menu(AIMenuStatus::TTTeve(None)),
                AIMenuStatus::UTTpve => self.go_to_utt_menu(),
                AIMenuStatus::UTTeve(None) => self.go_to_utt_menu(),
                AIMenuStatus::UTTeve(Some(_)) => self.go_to_ai_menu(AIMenuStatus::UTTeve(None)),
            },
            Scene::PlayingUTT(game) => game.input_esc(),
            Scene::PlayingTTT(_) => {}
        }
    }

    /// Handles 's' key input to allow AI to play first in PvE mode.
    pub fn handle_second(&mut self) {
        let online_connected = matches!(self.network_status, NetworkStatus::Connected { .. });
        let yielded = match &mut self.current_scene {
            Scene::PlayingTTT(game)
                if matches!(game.mode, GameMode::OnlinePvP(_)) && online_connected =>
            {
                game.yield_online_first_move()
            }
            Scene::PlayingTTT(game) if matches!(game.mode, GameMode::OnlinePvP(_)) => false,
            Scene::PlayingTTT(game) => {
                game.play_second();
                false
            }
            Scene::PlayingUTT(game) => {
                game.play_second();
                false
            }
            _ => false,
        };
        if yielded {
            self.send_online_action(NetworkCommand::YieldFirstMove);
        }
    }

    /// Handles 'c' key input to concede an active online game.
    pub fn handle_concede(&mut self) {
        let online_connected = matches!(self.network_status, NetworkStatus::Connected { .. });
        let conceded = match &mut self.current_scene {
            Scene::PlayingTTT(game)
                if matches!(game.mode, GameMode::OnlinePvP(_)) && online_connected =>
            {
                game.concede_online()
            }
            _ => false,
        };
        if conceded {
            self.send_online_action(NetworkCommand::Concede);
        }
    }

    /// Handles 'r' key input to reset the current game.
    pub fn handle_reset(&mut self) {
        let online_connected = matches!(self.network_status, NetworkStatus::Connected { .. });
        let requested = match &mut self.current_scene {
            Scene::PlayingTTT(game)
                if matches!(game.mode, GameMode::OnlinePvP(_)) && online_connected =>
            {
                game.request_online_rematch()
            }
            Scene::PlayingTTT(game) if matches!(game.mode, GameMode::OnlinePvP(_)) => false,
            Scene::PlayingTTT(game) => {
                game.reset_game();
                false
            }
            Scene::PlayingUTT(game) => {
                game.reset_game();
                false
            }
            _ => false,
        };
        if requested {
            self.send_online_action(NetworkCommand::SendRematchReady);
        }
    }

    /// Handles 'm' key input to return to main menu from game.
    pub fn handle_main_menu(&mut self) {
        let is_online = matches!(
            &self.current_scene,
            Scene::PlayingTTT(game) if matches!(game.mode, GameMode::OnlinePvP(_))
        );
        let is_game = matches!(
            self.current_scene,
            Scene::PlayingTTT(_) | Scene::PlayingUTT(_)
        );

        if is_online {
            self.stop_network();
        }
        if is_game {
            self.go_to_main_menu();
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
    use crate::game::{Board, GameState, Mark::X};

    #[test]
    fn test_app_new_starts_at_menu() {
        let app = App::new();
        assert!(matches!(app.current_scene, Scene::MainMenu(_)));
        assert!(!app.should_quit);
        assert_eq!(app.network_status, NetworkStatus::Idle);
        assert!(!app.network_is_active());
    }

    #[test]
    fn test_network_starts_and_stops_lazily() {
        let mut app = App::new();

        app.start_network().unwrap();
        assert!(app.network_is_active());

        app.stop_network();
        assert!(!app.network_is_active());
        assert_eq!(app.network_status, NetworkStatus::Idle);
    }

    #[test]
    fn test_online_command_starts_network_and_reports_invalid_ticket() {
        let mut app = App::new();

        app.join_online_match("invalid ticket".to_string(), GameVariant::Classic)
            .unwrap();
        assert!(app.network_is_active());

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        while std::time::Instant::now() < deadline {
            app.poll_network_events();
            if matches!(app.network_status, NetworkStatus::Failed(_)) {
                break;
            }
            std::thread::yield_now();
        }

        assert!(matches!(app.network_status, NetworkStatus::Failed(_)));
    }

    #[test]
    fn test_connected_event_starts_online_game_with_assigned_mark() {
        let mut app = App::new();

        app.handle_network_event(NetworkEvent::Connected {
            mark: X,
            game: GameVariant::Classic,
        });

        assert_eq!(app.network_status, NetworkStatus::Connected { mark: X });
        let Scene::PlayingTTT(game) = &app.current_scene else {
            panic!("expected online tic tac toe game");
        };
        assert_eq!(game.mode, GameMode::OnlinePvP(X));
    }

    #[test]
    fn test_connected_event_starts_online_ultimate_game() {
        let mut app = App::new();

        app.handle_network_event(NetworkEvent::Connected {
            mark: O,
            game: GameVariant::Ultimate,
        });

        assert_eq!(app.network_status, NetworkStatus::Connected { mark: O });
        let Scene::PlayingUTT(game) = &app.current_scene else {
            panic!("expected online ultimate tic tac toe game");
        };
        assert_eq!(game.mode, GameMode::OnlinePvP(O));
    }

    #[test]
    fn test_received_move_is_applied_to_online_game() {
        let mut app = App::new();
        app.handle_network_event(NetworkEvent::Connected {
            mark: O,
            game: GameVariant::Classic,
        });
        let message = MoveMessage::new(1, 2).unwrap();

        app.handle_network_event(NetworkEvent::MoveReceived(message));

        let Scene::PlayingTTT(game) = &app.current_scene else {
            panic!("expected online tic tac toe game");
        };
        assert_eq!(game.board.get(1, 2), Some(X));
        assert_eq!(game.active_player, O);
    }

    #[test]
    fn test_received_yield_gives_local_player_first_move() {
        let mut app = App::new();
        app.handle_network_event(NetworkEvent::Connected {
            mark: O,
            game: GameVariant::Classic,
        });

        app.handle_network_event(NetworkEvent::YieldFirstMoveReceived);

        let Scene::PlayingTTT(game) = &app.current_scene else {
            panic!("expected online tic tac toe game");
        };
        assert_eq!(game.active_player, O);
        assert_eq!(game.turn, 0);
    }

    #[test]
    fn test_received_rematch_readiness_completes_two_party_rematch() {
        let mut app = App::new();
        app.handle_network_event(NetworkEvent::Connected {
            mark: X,
            game: GameVariant::Classic,
        });
        let Scene::PlayingTTT(game) = &mut app.current_scene else {
            panic!("expected online tic tac toe game");
        };
        game.board.state = GameState::Won(X);
        game.turn = 5;
        assert!(game.request_online_rematch());

        app.handle_network_event(NetworkEvent::RematchReadyReceived);

        let Scene::PlayingTTT(game) = &app.current_scene else {
            panic!("expected online tic tac toe game");
        };
        assert_eq!(game.board.state, GameState::Playing);
        assert_eq!(game.active_player, O);
        assert_eq!(game.turn, 0);
    }

    #[test]
    fn test_received_concession_awards_local_player_the_win() {
        let mut app = App::new();
        app.handle_network_event(NetworkEvent::Connected {
            mark: X,
            game: GameVariant::Classic,
        });

        app.handle_network_event(NetworkEvent::OpponentConceded);

        let Scene::PlayingTTT(game) = &app.current_scene else {
            panic!("expected online tic tac toe game");
        };
        assert_eq!(game.board.state, GameState::Won(X));
    }

    #[test]
    fn test_opponent_disconnect_freezes_online_game() {
        let mut app = App::new();
        app.handle_network_event(NetworkEvent::Connected {
            mark: X,
            game: GameVariant::Classic,
        });
        app.handle_network_event(NetworkEvent::OpponentDisconnected);

        app.handle_right();
        app.handle_enter();
        app.handle_second();
        app.handle_reset();

        assert_eq!(app.network_status, NetworkStatus::OpponentDisconnected);
        let Scene::PlayingTTT(game) = &app.current_scene else {
            panic!("expected online tic tac toe game");
        };
        assert_eq!(game.selected.row, 0);
        assert_eq!(game.selected.col, 0);
        assert_eq!(game.turn, 0);
        assert_eq!(game.active_player, X);
        assert!(game.board.get(0, 0).is_none());
    }

    #[test]
    fn test_invalid_received_move_sets_failed_status() {
        let mut app = App::new();
        app.handle_network_event(NetworkEvent::Connected {
            mark: X,
            game: GameVariant::Classic,
        });

        app.handle_network_event(NetworkEvent::MoveReceived(MoveMessage::new(0, 0).unwrap()));

        assert!(matches!(app.network_status, NetworkStatus::Failed(_)));
        let Scene::PlayingTTT(game) = &app.current_scene else {
            panic!("expected online tic tac toe game");
        };
        assert!(game.board.get(0, 0).is_none());
    }

    #[test]
    fn test_opening_online_menu_does_not_start_network() {
        let mut app = App::new();
        app.go_to_ttt_menu();

        app.handle_enter();

        assert!(matches!(app.current_scene, Scene::OnlineTTTMenu(_)));
        assert!(!app.network_is_active());

        app.handle_esc();
        assert!(matches!(app.current_scene, Scene::TTTMenu(_)));
    }

    #[test]
    fn test_host_match_starts_and_cancels_network() {
        let mut app = App::new();
        app.go_to_online_ttt_menu();

        app.handle_enter();

        assert!(matches!(app.current_scene, Scene::HostingOnlineTTT));
        assert!(app.network_is_active());

        app.handle_esc();
        assert!(matches!(app.current_scene, Scene::OnlineTTTMenu(_)));
        assert!(!app.network_is_active());
        assert_eq!(app.network_status, NetworkStatus::Idle);
    }

    #[test]
    fn test_join_ticket_input_and_cancellation() {
        let mut app = App::new();
        app.go_to_online_ttt_menu();
        app.handle_down();
        app.handle_enter();

        assert!(matches!(app.current_scene, Scene::JoiningOnlineTTT(_)));
        assert!(!app.network_is_active());

        app.handle_enter();
        assert_eq!(
            app.network_status,
            NetworkStatus::Failed("ticket cannot be empty".to_string())
        );
        assert!(!app.network_is_active());

        assert!(app.handle_text_input("abcd efgh\nijkl\t"));
        assert_eq!(app.network_status, NetworkStatus::Idle);
        app.handle_backspace();
        let Scene::JoiningOnlineTTT(input) = &app.current_scene else {
            panic!("expected ticket input scene");
        };
        assert_eq!(input.value, "abcdefghijk");

        app.handle_esc();
        assert!(matches!(app.current_scene, Scene::OnlineTTTMenu(_)));
        assert!(!app.network_is_active());
        assert_eq!(app.network_status, NetworkStatus::Idle);
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
            game.play_move();
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
    fn test_main_menu_stops_online_network() {
        let mut app = App::new();
        app.start_network().unwrap();
        app.start_ttt_game(GameMode::OnlinePvP(X));

        app.handle_main_menu();

        assert!(matches!(app.current_scene, Scene::MainMenu(_)));
        assert!(!app.network_is_active());
        assert_eq!(app.network_status, NetworkStatus::Idle);
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
