use crate::app::App;
use crate::game::base::SmallBoard;
use crate::game::{Board, GameState, Mark};
use crate::scenes::{GameMode, GamePlayTTT, GamePlayUTT, Menu, Scene};
use crate::utils::Position;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

const PURPLE: Color = Color::Indexed(93);

/// Main render function that delegates to the appropriate screen renderer.
pub fn render(f: &mut Frame, app: &App) {
    match &app.current_scene {
        Scene::MainMenu(menu) | Scene::TTTMenu(menu) | Scene::UTTMenu(menu) => render_menu(f, menu),
        Scene::PlayingTTT(game) => render_game_ttt(f, game),
        Scene::PlayingUTT(game) => render_game_utt(f, game),
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
fn render_game_ttt(f: &mut Frame, game: &GamePlayTTT) {
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
    render_ttt_board(f, chunks[1], game);
    render_ttt_instructions(f, chunks[2], game);
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
fn render_ttt_board(f: &mut Frame, area: Rect, game: &GamePlayTTT) {
    let board_area = center_rect(area, 25, 9);

    let mut lines = vec![Line::from("")];

    // Add current player or game result
    let (status, status_style) = game_status(game.board.state, game.active_player);

    // Render the board
    for y in 0..5 {
        lines.push(ttt_board_line(
            &game.board,
            y,
            Some((game.selected, game.active_player)),
            Style::default(),
        ));
    }

    let game_block = Block::default()
        .borders(Borders::ALL)
        .title(status)
        .title_style(status_style);

    let board = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(game_block);

    f.render_widget(board, board_area);
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
                                .fg(Color::Yellow)
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
fn render_ttt_instructions(f: &mut Frame, area: Rect, game: &GamePlayTTT) {
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

fn game_status(game_state: GameState, current_player: Mark) -> (String, Style) {
    match game_state {
        GameState::Playing => (
            format!("Current Player: {}", current_player),
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
            Style::default().fg(PURPLE).add_modifier(Modifier::BOLD),
        ),
    }
}

/// Renders the Ultimate Tic-Tac-Toe game screen.
fn render_game_utt(f: &mut Frame, game: &GamePlayUTT) {
    if render_size_warning(f, 20, 20) {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Max(3),
            Constraint::Min(25),
            Constraint::Length(4),
        ])
        .split(f.area());

    let mode_name = match game.mode {
        GameMode::PvE => "Play vs AI",
        GameMode::LocalPvP => "Local PvP",
    };

    render_game_mode(mode_name, f, chunks[0]);
    render_utt_board(f, chunks[1], game);
    render_utt_instructions(f, chunks[2], game);
}

/// Renders the Ultimate Tic-Tac-Toe board.
fn render_utt_board(f: &mut Frame, area: Rect, game: &GamePlayUTT) {
    let board_area = center_rect(area, 55, 23);

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
                let small_board = game.big_board.get_board(big_row, big_col);
                let (selection, board_style) = if game.selected_board.row == big_row
                    && game.selected_board.col == big_col
                    && game.big_board.state == GameState::Playing
                {
                    match game.selected_cell {
                        Some(position) => (
                            Some((position, game.active_player)),
                            Style::default().fg(Color::Cyan),
                        ),
                        None => (None, Style::default().fg(Color::Yellow)),
                    }
                } else {
                    match small_board.state {
                        GameState::Playing => (None, Style::default()),
                        GameState::Draw => (None, Style::default().fg(PURPLE)),
                        GameState::Won(Mark::X) => (None, Style::default().fg(Color::Red)),
                        GameState::Won(Mark::O) => (None, Style::default().fg(Color::Blue)),
                    }
                };

                y_spans
                    .append(&mut ttt_board_line(small_board, small_y, selection, board_style).spans)
            }
            lines.push(Line::from(y_spans));
        }
    }

    let game_block = Block::default()
        .borders(Borders::ALL)
        .title(status)
        .title_style(status_style);

    let board = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(game_block);

    f.render_widget(board, board_area);
}

/// Renders instructions for Ultimate Tic-Tac-Toe.
fn render_utt_instructions(f: &mut Frame, area: Rect, game: &GamePlayUTT) {
    let instructions = if game.big_board.state == GameState::Playing {
        if game.selected_cell.is_none() {
            vec![
                "Arrow Keys: Select Board | Enter: Select Cell".to_string(),
                "R: Reset Game | M: Main Menu | Q: Quit".to_string(),
            ]
        } else {
            vec![
                "Arrow Keys: Select Cell | Enter: Place Mark".to_string(),
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
