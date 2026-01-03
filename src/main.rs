use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;
use tic_tac_foe::app::{App, CurrentScreen};
use tic_tac_foe::ui;

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

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>>
where
    B::Error: 'static,
{
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        match app.current_screen {
            CurrentScreen::SelectingGameMode => {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            app.quit();
                        }
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            app.start_local_pve();
                        }
                        KeyCode::Char('l') | KeyCode::Char('L') => {
                            app.start_local_pvp();
                        }
                        _ => {}
                    }
                }
            }
            CurrentScreen::Playing => {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            app.quit();
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            app.reset_game();
                        }
                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            app.reset_app();
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.input_left();
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            app.input_right();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.input_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.input_down();
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            app.make_move();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
