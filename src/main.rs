use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::Terminal;
use tic_tac_foe::app::App;
use tic_tac_foe::ui;

/// Entry point for the Tic-Tac-Toe TUI application.
///
/// Initializes the terminal, runs the main event loop, and properly
/// restores the terminal state on exit.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();

    let mut app = App::new();

    let result = run_app(&mut terminal, &mut app);

    ratatui::restore();

    if let Err(err) = result {
        println!("Error: {:?}", err);
    }

    Ok(())
}

/// Main application loop that handles rendering and input events.
///
/// Continuously draws the UI and processes keyboard input until the
/// user quits the application.
///
/// # Arguments
/// * `terminal` - The terminal backend to render to
/// * `app` - The application state
///
/// # Returns
/// Result indicating success or any error encountered
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>>
where
    B::Error: 'static,
{
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        app.quit();
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        app.handle_reset();
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        app.handle_main_menu();
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        app.handle_second();
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        app.handle_left();
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        app.handle_right();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.handle_up();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.handle_down();
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        app.handle_enter();
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
