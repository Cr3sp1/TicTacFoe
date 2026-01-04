use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::{App, CurrentScreen};
use crate::game::Mark;
use crate::scenes::{GameMode, GamePlay, GameState, MainMenu};

pub fn render(f: &mut Frame, app: &App) {
    match &app.current_screen {
        CurrentScreen::MainMenu(menu) => render_main_menu(f, menu),
        CurrentScreen::Playing(game) => render_game(f, game),
    }
}

fn render_game(f: &mut Frame, game: &GamePlay) {
    if render_size_warning(f, 10, 10){
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Max(3),
            Constraint::Min(11),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_title(f, chunks[0]);
    render_board(f, chunks[1], game);
    render_game_instructions(f, chunks[2], game);
}

fn render_main_menu(f: &mut Frame, menu: &MainMenu) {
    if render_size_warning(f, 10, 10){
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_title(f, chunks[0]);
    render_menu_options(f, chunks[1], menu);
    render_menu_instructions(f, chunks[2]);
}

fn render_menu_options(f: &mut Frame, area: Rect, menu: &MainMenu) {
    let options_area = center_rect(area, 30, 10);

    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Select Game Mode:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

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

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(paragraph, options_area);
}

fn render_menu_instructions(f: &mut Frame, area: Rect) {
    let instructions = "Arrow Keys: Navigate | Enter: Select | Q: Quit";

    render_instructions(f, area, instructions);
}

fn render_title(f: &mut Frame, area: Rect) {
    let title_area = center_rect(area, 100, 3);
    let title = Paragraph::new("TIC TAC FOE")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, title_area);
}

fn render_board(f: &mut Frame, area: Rect, game: &GamePlay) {
    let board_area = center_rect(area, 40, 11);

    let mut lines = vec![];

    // Add current player or game result
    match game.state {
        GameState::Playing => {
            lines.push(Line::from(vec![Span::styled(
                format!("Current Player: {}", game.active_player),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
        }
        GameState::Won(winner) => {
            lines.push(Line::from(vec![Span::styled(
                format!("Player {} WINS!", winner),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]));
        }
        GameState::Draw => {
            lines.push(Line::from(vec![Span::styled(
                "DRAW!",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
        }
    }
    lines.push(Line::from(""));

    // Render the board
    for row in 0..3 {
        let mut row_spans = vec![];
        for col in 0..3 {
            let mut cell_content = match game.board.get(row, col) {
                Some(Mark::X) => "X",
                Some(Mark::O) => "O",
                None => " ",
            };

            let style = if row == game.selected_row
                && col == game.selected_col
                && game.state == GameState::Playing
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

    let board = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(board, board_area);
}

fn render_game_instructions(f: &mut Frame, area: Rect, game: &GamePlay) {
    let instructions = if game.state == GameState::Playing {
        if game.turn == 0 && game.mode == GameMode::PvE {
            "S: Play Second | Arrow Keys: Move | Enter: Place Mark | R: Reset Game | M: Main Menu | Q: Quit"
        } else {
            "Arrow Keys: Move | Enter: Place Mark | R: Reset Game | M: Main Menu | Q: Quit"
        }
    } else {
        "R: Reset Game | M: Main Menu | Q: Quit"
    };

    render_instructions(f, area, instructions);
}

fn render_instructions(f: &mut Frame, area: Rect, instructions: &str) {
    let width = instructions.chars().count() as u16 + 5;
    let area = center_rect(area, width, 3);

    let paragraph = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Commands"));

    f.render_widget(paragraph, area);
}

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

fn render_size_warning(f: &mut Frame, min_width: u16, min_height: u16) -> bool {
    let size = f.area();
    if size.width >= min_width && size.height >= min_height {
        return false;
    }

    let warning = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "Terminal Too Small!",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(format!("Minimum required: {}x{}", min_width, min_height)),
        Line::from(format!("Current size: {}x{}", size.width, size.height)),
        Line::from("Please resize your terminal"),
    ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Warning"));

    f.render_widget(warning, size);
    true
}
