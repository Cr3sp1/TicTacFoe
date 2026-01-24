use crate::app::App;
use crate::game::{Board, GameState, Mark};
use crate::scenes::{GameMode, GamePlayTTT, Menu, Scene};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

/// Main render function that delegates to the appropriate screen renderer.
pub fn render(f: &mut Frame, app: &App) {
    match &app.current_scene {
        Scene::MainMenu(menu) | Scene::TTTMenu(menu) | Scene::UTTMenu(menu) => render_menu(f, menu),
        Scene::PlayingTTT(game) => render_game(f, game),
    }
}

/// Renders a menu screen with game mode options.
fn render_menu(f: &mut Frame, menu: &Menu) {
    if render_size_warning(f, 10, 10) {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_title(f, chunks[0]);
    render_menu_options(f, chunks[1], menu);
    render_menu_instructions(f, chunks[2]);
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
fn render_menu_options(f: &mut Frame, area: Rect, menu: &Menu) {
    let options_area = center_rect(area, 25, 9);

    let mut lines = vec![Line::from("")];

    for (i, option) in menu.options.iter().enumerate() {
        let style = if i == menu.selected_option {
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {}  ", option),
            style,
        )]));
        lines.push(Line::from(""));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Select Game Mode")
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(block);

    f.render_widget(paragraph, options_area);
}

/// Renders the instruction text for the main menu.
fn render_menu_instructions(f: &mut Frame, area: Rect) {
    let instructions = &["Arrow Keys: Navigate | Enter: Select | Q: Quit".to_string()];

    render_instructions(f, area, instructions);
}

/// Renders the game screen with board and status.
fn render_game(f: &mut Frame, game: &GamePlayTTT) {
    if render_size_warning(f, 10, 10) {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Max(3),
            Constraint::Min(11),
            Constraint::Length(4),
        ])
        .split(f.area());

    let mode_name = match game.mode {
        GameMode::PvE => "Play vs AI",
        GameMode::LocalPvP => "Local PvP",
    };

    render_game_mode(mode_name, f, chunks[0]);
    render_board(f, chunks[1], game);
    render_game_instructions(f, chunks[2], game);
}

/// Renders the current game mode indicator.
fn render_game_mode(mode_name: &str, f: &mut Frame, area: Rect) {
    let width = mode_name.chars().count() as u16 + 5;
    let title_area = center_rect(area, width, 3);
    let title = Paragraph::new(mode_name)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::HeavyDoubleDashed)
                .title("Mode"),
        );
    f.render_widget(title, title_area);
}

/// Renders the tic-tac-toe board with current marks and selection highlight.
fn render_board(f: &mut Frame, area: Rect, game: &GamePlayTTT) {
    let board_area = center_rect(area, 25, 9);

    let mut lines = vec![Line::from("")];

    // Add current player or game result
    let (status, style) = match game.board.state {
        GameState::Playing => (
            format!("Current Player: {}", game.active_player),
            Style::default()
                .fg(Color::Yellow)
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
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    };

    // Render the board
    for row in 0..3 {
        let mut row_spans = vec![];
        for col in 0..3 {
            let mut cell_content = match game.board.get(row, col) {
                Some(Mark::X) => "X",
                Some(Mark::O) => "O",
                None => " ",
            };

            let style = if row == game.selected.row
                && col == game.selected.col
                && game.board.state == GameState::Playing
            {
                cell_content = match game.active_player {
                    Mark::X => "X",
                    Mark::O => "O",
                };
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                match game.board.get(row, col) {
                    Some(Mark::X) => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    Some(Mark::O) => Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                    None => Style::default().fg(Color::DarkGray),
                }
            };

            row_spans.push(Span::styled(format!(" {} ", cell_content), style));

            if col < 2 {
                row_spans.push(Span::raw("|"));
            }
        }
        lines.push(Line::from(row_spans));

        if row < 2 {
            lines.push(Line::from("-----------"));
        }
    }

    let game_block = Block::default()
        .borders(Borders::ALL)
        .title(status)
        .title_style(style);

    let board = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(game_block);

    f.render_widget(board, board_area);
}

/// Renders context-appropriate instructions for the game screen.
fn render_game_instructions(f: &mut Frame, area: Rect, game: &GamePlayTTT) {
    let instructions = if game.board.state == GameState::Playing {
        if game.turn == 0 && game.mode == GameMode::PvE {
            vec![
                "S: Play Second | Arrow Keys: Move | Enter: Place Mark".to_string(),
                "R: Reset Game | M: Main Menu | Q: Quit".to_string(),
            ]
        } else {
            vec![
                "Arrow Keys: Move | Enter: Place Mark".to_string(),
                "R: Reset Game | M: Main Menu | Q: Quit".to_string(),
            ]
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
                .title("Commands"),
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
