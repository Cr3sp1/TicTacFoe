use crate::app::App;
use crate::game::base::SmallBoard;
use crate::game::{Board, GameState, Mark};
use crate::network::NetworkStatus;
use crate::scenes::{AIMenuStatus, GameMode, GamePlayTTT, GamePlayUTT, Menu, Scene, TicketInput};
use crate::utils::Position;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

const PURPLE: Color = Color::Indexed(93);
const TICKET_GROUP_SIZE: usize = 4;
const TICKET_GROUPS_PER_LINE: usize = 6;
const TICKET_LINE_WIDTH: u16 =
    (TICKET_GROUP_SIZE * TICKET_GROUPS_PER_LINE + TICKET_GROUPS_PER_LINE - 1) as u16;

/// Main render function that delegates to the appropriate screen renderer.
pub fn render(f: &mut Frame, app: &App) {
    match &app.current_scene {
        Scene::MainMenu(menu) => render_menu(f, menu, "Select Game"),
        Scene::TTTMenu(menu) | Scene::UTTMenu(menu) => render_menu(f, menu, "Select Game Mode"),
        Scene::OnlineMenu(menu, _) => render_menu(f, menu, "Select Connection Method"),
        Scene::HostingOnline(_) => render_hosting_ttt(f, &app.network_status),
        Scene::JoiningOnline(input, _) => render_joining_ttt(f, input, &app.network_status),
        Scene::AIMenu(menu, status) => render_menu(f, menu, ai_menu_title(status)),
        Scene::PlayingTTT(game) => render_game_ttt(f, game, &app.network_status),
        Scene::PlayingUTT(game) => render_game_utt(f, game),
    }
}

/// Renders the main menu screen with game options.
fn render_menu(f: &mut Frame, menu: &Menu, title: &str) {
    if render_size_warning(f, 14, 13) {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Max(7),
            Constraint::Min(13),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_title(f, chunks[0]);
    render_menu_options(f, chunks[1], menu, title);
    render_menu_instructions(f, chunks[2]);
}

fn render_hosting_ttt(f: &mut Frame, status: &NetworkStatus) {
    if render_size_warning(f, 33, 12) {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(7),
            Constraint::Min(13),
            Constraint::Length(3),
        ])
        .split(f.area());
    render_title(f, chunks[0]);

    let width = chunks[1].width.saturating_sub(4).min(90);
    let area = center_rect(chunks[1], width, 13);
    let content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(area);

    let (message, style) = match status {
        NetworkStatus::Idle => ("Starting network...", Style::default().fg(Color::Yellow)),
        NetworkStatus::Hosting { relay_ready, .. } => (
            if *relay_ready {
                "LAN and internet ready"
            } else {
                "LAN ready - preparing internet connectivity"
            },
            Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        ),
        NetworkStatus::Connecting => ("Connecting...", Style::default().fg(Color::Yellow)),
        NetworkStatus::Connected { .. } => (
            "Opponent connected",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        NetworkStatus::OpponentDisconnected => (
            "Opponent disconnected",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        NetworkStatus::Failed(_) => (
            "Unable to host match",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    };
    let header_area = center_rect(content[0], 60, 3);

    let header = Paragraph::new(message)
        .style(style)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Host Online Match"),
        );
    f.render_widget(header, header_area);

    match status {
        NetworkStatus::Hosting { ticket, .. } => {
            f.render_widget(
                Paragraph::new("Copy and share this ticket:")
                    .alignment(Alignment::Center)
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                content[1],
            );
            let ticket = format_ticket_lines(ticket)
                .into_iter()
                .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::LightYellow))))
                .collect::<Vec<_>>();
            let ticket_area = center_rect(content[2], 32, 8);
            f.render_widget(
                Paragraph::new(ticket).alignment(Alignment::Center),
                ticket_area,
            );
        }
        NetworkStatus::Failed(error) => {
            f.render_widget(
                Paragraph::new(format!("Error: {error}"))
                    .style(Style::default().fg(Color::Red))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false }),
                content[2],
            );
        }
        _ => {}
    }

    render_instructions(f, chunks[2], &["Esc: Cancel | Q: Quit".to_string()]);
}

fn render_joining_ttt(f: &mut Frame, input: &TicketInput, status: &NetworkStatus) {
    if render_size_warning(f, 40, 12) {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(7),
            Constraint::Min(13),
            Constraint::Length(3),
        ])
        .split(f.area());
    render_title(f, chunks[0]);

    let width = chunks[1].width.saturating_sub(4).min(90);
    let area = center_rect(chunks[1], width, 13);
    let content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(1),
        ])
        .split(area);

    let (message, style) = match status {
        NetworkStatus::Idle => ("Enter host ticket", Style::default().fg(Color::LightYellow)),
        NetworkStatus::Connecting => ("Connecting...", Style::default().fg(Color::Yellow)),
        NetworkStatus::Connected { .. } => (
            "Connected",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        NetworkStatus::OpponentDisconnected => (
            "Opponent disconnected",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        NetworkStatus::Failed(_) => (
            "Unable to join match",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        NetworkStatus::Hosting { .. } => ("Enter host ticket", Style::default()),
    };
    let header_area = center_rect(content[0], 60, 3);
    f.render_widget(
        Paragraph::new(message)
            .style(style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Join Online Match"),
            ),
        header_area,
    );

    let formatted_ticket = format_ticket_lines(&input.value).join("\n");
    let ticket_area = center_rect(
        content[1],
        (TICKET_LINE_WIDTH + 4).min(content[1].width),
        content[1].height,
    );
    f.render_widget(
        Paragraph::new(formatted_ticket)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Ticket")),
        ticket_area,
    );

    if let NetworkStatus::Failed(error) = status {
        f.render_widget(
            Paragraph::new(format!("Error: {error}"))
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center),
            content[2],
        );
    }

    render_instructions(
        f,
        chunks[2],
        &["Paste ticket | Enter: Connect | Esc: Cancel".to_string()],
    );
}

fn format_ticket_lines(ticket: &str) -> Vec<String> {
    let characters = ticket.chars().collect::<Vec<_>>();
    let groups = characters
        .chunks(TICKET_GROUP_SIZE)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>();

    groups
        .chunks(TICKET_GROUPS_PER_LINE)
        .map(|line| line.join(" "))
        .collect::<Vec<_>>()
}

fn ai_menu_title(status: &AIMenuStatus) -> &str {
    match status {
        AIMenuStatus::TTTpve | AIMenuStatus::UTTpve => "Select AI",
        AIMenuStatus::TTTeve(None) | AIMenuStatus::UTTeve(None) => "Select AI for X",
        AIMenuStatus::TTTeve(Some(_)) | AIMenuStatus::UTTeve(Some(_)) => "Select AI for O",
    }
}

/// Renders the ASCII art title banner.
fn render_title(f: &mut Frame, area: Rect) {
    let title_area = center_rect(area, 72, 7);

    let ascii_art = vec![
        "OOXXOO  XXOX   OXOO    XOXOXO   XOX    XOOX    OOXXO   XOO   XXOOX",
        "  OO     XO   X          XO    X   X  X        XX     O   O  OO   ",
        "  XO     OX   X          OX    XOOXO  O        OOXX   X   O  OXOO ",
        "  OX     OX   O          OO    O   X  O        XO     O   X  XO   ",
        "  OO    XXOX   OXXO      XO    X   O   XXOO    OX      OXO   XOXOX",
    ];

    let lines: Vec<Line> = ascii_art
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                *line,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    let title = Paragraph::new(lines)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double),
        );
    f.render_widget(title, title_area);
}

/// Renders the menu options with highlighting for the selected option.
fn render_menu_options(f: &mut Frame, area: Rect, menu: &Menu, title: &str) {
    let options_area = center_rect(area, 30, 13);

    let mut lines = vec![Line::from("")];

    for (i, option) in menu.options.iter().enumerate() {
        let (style, prefix) = if i == menu.selected_option {
            (
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::LightYellow),
                "  🢒 ",
            )
        } else {
            (Style::default().add_modifier(Modifier::BOLD), "    ")
        };

        lines.push(
            Line::from(vec![Span::styled(format!("{}{}  ", prefix, option), style)]).left_aligned(),
        );
        lines.push(Line::from(""));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        );

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(block);

    f.render_widget(paragraph, options_area);
}

/// Renders the instruction text for the main menu.
fn render_menu_instructions(f: &mut Frame, area: Rect) {
    let instructions = &["Arrow Keys: Navigate | Enter: Select | Esc: Back | Q: Quit".to_string()];

    render_instructions(f, area, instructions);
}

/// Renders the game screen with board and status.
fn render_game_ttt(f: &mut Frame, game: &GamePlayTTT, network_status: &NetworkStatus) {
    if render_size_warning(f, 10, 10) {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Max(7),
            Constraint::Min(9),
            Constraint::Length(4),
        ])
        .split(f.area());

    render_title(f, chunks[0]);
    render_ttt_board(f, chunks[1], game, network_status);
    render_ttt_instructions(f, chunks[2], game, network_status);
}

/// Renders the tic-tac-toe board with current marks and selection highlight.
fn render_ttt_board(f: &mut Frame, area: Rect, game: &GamePlayTTT, network_status: &NetworkStatus) {
    let board_area = center_rect(area, 25, 9);

    let mut lines = vec![Line::from("")];

    // Add current player or game result
    let (status, status_style) = ttt_game_status(game, network_status);

    let selection = match (&game.mode, network_status) {
        (GameMode::OnlinePvP(_), NetworkStatus::OpponentDisconnected) => None,
        _ => match game.mode {
            GameMode::EvE(_, _) => None,
            GameMode::OnlinePvP(local_mark) if local_mark != game.active_player => None,
            _ => Some((game.selected, game.active_player)),
        },
    };

    // Render the board
    for y in 0..5 {
        lines.push(ttt_board_line(&game.board, y, selection, Style::default()));
    }

    let mode_name = match game.mode {
        GameMode::PvE(_) => "Mode: Play vs AI",
        GameMode::EvE(_, _) => "Mode: AI vs AI",
        GameMode::LocalPvP => "Mode: Local PvP",
        GameMode::OnlinePvP(_) => "Mode: Online PvP",
    };

    let game_block = game_block(mode_name, status.as_str(), status_style);

    let board = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(game_block);

    f.render_widget(board, board_area);
}

fn game_block<'a>(mode_name: &'a str, status: &'a str, status_style: Style) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(
            Line::from(status)
                .style(status_style)
                .alignment(Alignment::Center),
        )
        .title_style(status_style)
        .title_bottom(
            Line::from(mode_name)
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center),
        )
}

fn ttt_board_line(
    board: &SmallBoard,
    y: usize,
    selection: Option<(Position, Mark)>,
    board_style: Style,
) -> Line<'static> {
    let row = match y {
        val if val >= 5 => {
            panic!("Invalid value of y, must be in [0, 4].")
        }
        even if even % 2 == 0 => even / 2,
        _ => {
            return Line::from(Span::raw("───┼───┼───").style(board_style));
        }
    };
    let mut row_spans = vec![];
    for col in 0..3 {
        let (cell_content, style) = match board.get(row, col) {
            Some(Mark::X) => (
                "X",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Some(Mark::O) => (
                "O",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            None => {
                if let Some((position, mark)) = selection {
                    if row == position.row
                        && col == position.col
                        && board.state == GameState::Playing
                    {
                        let display = match mark {
                            Mark::X => "X",
                            Mark::O => "O",
                        };
                        (
                            display,
                            Style::default()
                                .fg(Color::LightYellow)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        (" ", Style::default())
                    }
                } else {
                    (" ", Style::default())
                }
            }
        };

        row_spans.push(Span::styled(format!(" {} ", cell_content), style));

        if col < 2 {
            row_spans.push(Span::raw("│").style(board_style));
        }
    }
    Line::from(row_spans)
}

/// Renders context-appropriate instructions for the game screen.
fn render_ttt_instructions(
    f: &mut Frame,
    area: Rect,
    game: &GamePlayTTT,
    network_status: &NetworkStatus,
) {
    let instructions = if matches!(game.mode, GameMode::OnlinePvP(_))
        && matches!(network_status, NetworkStatus::OpponentDisconnected)
    {
        vec!["M: Main Menu | Q: Quit".to_string()]
    } else if matches!(game.mode, GameMode::OnlinePvP(_)) && game.waiting_for_rematch() {
        vec![
            "Waiting for opponent".to_string(),
            "M: Main Menu | Q: Quit".to_string(),
        ]
    } else if game.board.state == GameState::Playing {
        match game.mode {
            GameMode::OnlinePvP(local_mark) if local_mark != game.active_player => vec![
                "Waiting for opponent".to_string(),
                "C: Concede | M: Main Menu | Q: Quit".to_string(),
            ],
            GameMode::OnlinePvP(_) if game.turn == 0 => vec![
                "S: Let Opponent Move First | Arrow Keys: Move".to_string(),
                "Enter: Place Mark | C: Concede | M: Main Menu | Q: Quit".to_string(),
            ],
            GameMode::OnlinePvP(_) => vec![
                "Arrow Keys: Move | Enter: Place Mark".to_string(),
                "C: Concede | M: Main Menu | Q: Quit".to_string(),
            ],
            GameMode::EvE(_, _) => {
                if game.turn == 0 {
                    vec![
                        "S: Let O Move First | Enter: Let Active AI play".to_string(),
                        "R: Reset Game | M: Main Menu | Q: Quit".to_string(),
                    ]
                } else {
                    vec![
                        "Enter: Let Active AI play".to_string(),
                        "R: Reset Game | M: Main Menu | Q: Quit".to_string(),
                    ]
                }
            }
            _ => {
                if game.turn != 0 || game.mode == GameMode::LocalPvP {
                    vec![
                        "Arrow Keys: Move | Enter: Place Mark".to_string(),
                        "R: Reset Game | M: Main Menu | Q: Quit".to_string(),
                    ]
                } else {
                    vec![
                        "S: Play Second | Arrow Keys: Move | Enter: Place Mark".to_string(),
                        "R: Reset Game | M: Main Menu | Q: Quit".to_string(),
                    ]
                }
            }
        }
    } else {
        vec!["R: Reset Game | M: Main Menu | Q: Quit".to_string()]
    };

    render_instructions(f, area, &instructions);
}

fn ttt_game_status(game: &GamePlayTTT, network_status: &NetworkStatus) -> (String, Style) {
    if matches!(game.mode, GameMode::OnlinePvP(_))
        && matches!(network_status, NetworkStatus::OpponentDisconnected)
    {
        return (
            "Opponent left the game".to_string(),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        );
    }
    let (status, style) = game_status(game.board.state, game.active_player);
    if game.board.state == GameState::Playing
        && matches!(
            game.mode,
            GameMode::OnlinePvP(local_mark) if local_mark != game.active_player
        )
    {
        (status, Style::default())
    } else {
        (status, style)
    }
}

fn game_status(game_state: GameState, current_player: Mark) -> (String, Style) {
    match game_state {
        GameState::Playing => (
            format!("Current Player: {}", current_player),
            Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        ),
        GameState::Won(Mark::X) => (
            "Player X WINS!".to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        GameState::Won(Mark::O) => (
            "Player O WINS!".to_string(),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        GameState::Draw => (
            "DRAW!".to_string(),
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
    }
}

/// Renders the Ultimate Tic-Tac-Toe game screen.
fn render_game_utt(f: &mut Frame, game: &GamePlayUTT) {
    if render_size_warning(f, 43, 20) {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        // .margin(0)
        .constraints([
            Constraint::Max(7),
            Constraint::Min(21),
            Constraint::Length(4),
        ])
        .split(f.area());

    render_title(f, chunks[0]);
    render_utt_board(f, chunks[1], game);
    render_utt_instructions(f, chunks[2], game);
}

/// Renders the Ultimate Tic-Tac-Toe board.
fn render_utt_board(f: &mut Frame, area: Rect, game: &GamePlayUTT) {
    let board_area = center_rect(area, 47, 21);

    let mut lines = vec![Line::from("")];

    // Add current player or game result
    let (status, status_style) = game_status(game.big_board.state, game.active_player);

    // Render the meta-board (3x3 grid of small boards)
    for big_y in 0..5 {
        let big_row = match big_y {
            even if even % 2 == 0 => even / 2,
            _ => {
                let mut y_spans: Vec<Span> = Vec::new();
                y_spans.push(Span::from("━".repeat(12)));
                y_spans.push(Span::from("╋"));
                y_spans.push(Span::from("━".repeat(13)));
                y_spans.push(Span::from("╋"));
                y_spans.push(Span::from("━".repeat(12)));
                lines.push(Line::from(y_spans));
                continue;
            }
        };
        for small_y in 0..5 {
            let mut y_spans: Vec<Span> = Vec::new();
            for big_x in 0..5 {
                let big_col = match big_x {
                    even if even % 2 == 0 => even / 2,
                    _ => {
                        y_spans.push(Span::raw(" ┃ "));
                        continue;
                    }
                };
                let (selection, board_style) = small_board_selection_style(game, big_row, big_col);
                let small_board = game.big_board.get_board(big_row, big_col);

                y_spans
                    .append(&mut ttt_board_line(small_board, small_y, selection, board_style).spans)
            }
            lines.push(Line::from(y_spans));
        }
    }

    let mode_name = match game.mode {
        GameMode::PvE(_) => "Mode: Play vs AI",
        GameMode::EvE(_, _) => "Mode: AI vs AI",
        GameMode::LocalPvP => "Mode: Local PvP",
        GameMode::OnlinePvP(_) => "Mode: Online PvP",
    };

    let game_block = game_block(mode_name, status.as_str(), status_style);

    let board = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(game_block);

    f.render_widget(board, board_area);
}

fn small_board_selection_style(
    game: &GamePlayUTT,
    big_row: usize,
    big_col: usize,
) -> (Option<(Position, Mark)>, Style) {
    let small_board = game.big_board.get_board(big_row, big_col);
    if matches!(game.mode, GameMode::EvE(_, _)) {
        match small_board.state {
            GameState::Draw => (None, Style::default().fg(PURPLE)),
            GameState::Won(Mark::X) => (None, Style::default().fg(Color::Red)),
            GameState::Won(Mark::O) => (None, Style::default().fg(Color::Blue)),
            GameState::Playing => match game.big_board.active_board {
                Some(selected) if selected == (big_row, big_col) => {
                    (None, Style::default().fg(Color::Green))
                }
                _ => (None, Style::default()),
            },
        }
    } else {
        if game.selected_board.row == big_row
            && game.selected_board.col == big_col
            && game.big_board.state == GameState::Playing
        {
            match game.selected_cell {
                Some(position) => (
                    Some((position, game.active_player)),
                    Style::default().fg(Color::Green),
                ),
                None => (None, Style::default().fg(Color::LightYellow)),
            }
        } else {
            match small_board.state {
                GameState::Playing => (None, Style::default()),
                GameState::Draw => (None, Style::default().fg(PURPLE)),
                GameState::Won(Mark::X) => (None, Style::default().fg(Color::Red)),
                GameState::Won(Mark::O) => (None, Style::default().fg(Color::Blue)),
            }
        }
    }
}

/// Renders instructions for Ultimate Tic-Tac-Toe.
fn render_utt_instructions(f: &mut Frame, area: Rect, game: &GamePlayUTT) {
    let instructions = if game.big_board.state == GameState::Playing {
        match game.mode {
            GameMode::EvE(_, _) => {
                if game.turn == 0 {
                    vec![
                        "S: Let O Move First | Enter: Let Active AI play".to_string(),
                        "R: Reset Game | M: Main Menu | Q: Quit".to_string(),
                    ]
                } else {
                    vec![
                        "Enter: Let Active AI play".to_string(),
                        "R: Reset Game | M: Main Menu | Q: Quit".to_string(),
                    ]
                }
            }
            _ => {
                if game.selected_cell.is_none() {
                    vec![
                        "Arrow Keys: Select Board | Enter: Confirm Board".to_string(),
                        "R: Reset Game | M: Main Menu | Q: Quit".to_string(),
                    ]
                } else {
                    if game.big_board.active_board.is_none() {
                        vec![
                            "Arrow Keys: Select Cell | Enter: Place Mark".to_string(),
                            "Esc: Change Board | R: Reset Game | M: Main Menu | Q: Quit"
                                .to_string(),
                        ]
                    } else {
                        vec![
                            "Arrow Keys: Select Cell | Enter: Place Mark".to_string(),
                            "R: Reset Game | M: Main Menu | Q: Quit".to_string(),
                        ]
                    }
                }
            }
        }
    } else {
        vec!["R: Reset Game | M: Main Menu | Q: Quit".to_string()]
    };

    render_instructions(f, area, &instructions);
}

/// Renders instruction text in a centered, bordered box.
///
/// # Arguments
/// * `f` - The frame to render to
/// * `area` - The available area
/// * `instructions` - Lines of instruction text to display
fn render_instructions(f: &mut Frame, area: Rect, instructions: &[String]) {
    let max_width = instructions
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0) as u16
        + 4;

    let height = instructions.len() as u16 + 2; // +2 for borders
    let area = center_rect(area, max_width, height);

    let lines: Vec<Line> = instructions
        .iter()
        .map(|s| Line::from(s.as_str()))
        .collect();

    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(
                    Line::from("Commands")
                        .centered()
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                ),
        );

    f.render_widget(paragraph, area);
}

/// Centers a rectangle of given dimensions within the provided area.
fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Length((area.height.saturating_sub(height)) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Length((area.width.saturating_sub(width)) / 2),
        ])
        .split(vertical[1])[1]
}

/// Renders a warning if the terminal is too small.
///
/// # Returns
/// `true` if warning was displayed, `false` if terminal size is adequate
fn render_size_warning(f: &mut Frame, min_width: u16, min_height: u16) -> bool {
    let size = f.area();
    if size.width >= min_width && size.height >= min_height {
        return false;
    }

    let warning = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "Terminal Too Small!",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from(format!("Minimum required: {}x{}", min_width, min_height)),
        Line::from(format!("Current size: {}x{}", size.width, size.height)),
        Line::from("Please resize your terminal"),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).title("Warning"));

    f.render_widget(warning, size);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_online_turn_status_uses_local_and_opponent_colors() {
        let mut game = GamePlayTTT::new(GameMode::OnlinePvP(Mark::O));

        let (_, opponent_style) =
            ttt_game_status(&game, &NetworkStatus::Connected { mark: Mark::O });
        assert_eq!(opponent_style, Style::default());

        game.active_player = Mark::O;
        let (_, local_style) = ttt_game_status(&game, &NetworkStatus::Connected { mark: Mark::O });
        assert_eq!(local_style.fg, Some(Color::LightYellow));
    }

    #[test]
    fn test_disconnected_opponent_replaces_online_game_status() {
        let game = GamePlayTTT::new(GameMode::OnlinePvP(Mark::X));

        let (status, style) = ttt_game_status(&game, &NetworkStatus::OpponentDisconnected);

        assert_eq!(status, "Opponent left the game");
        assert_eq!(style.fg, Some(Color::Magenta));
    }

    #[test]
    fn test_format_ticket_lines_groups_ticket_for_copying() {
        let lines = format_ticket_lines("abcdefghijklmnopqrstuvwxy");

        assert_eq!(
            lines,
            vec!["abcd efgh ijkl mnop qrst uvwx".to_string(), "y".to_string(),]
        );
    }
}
