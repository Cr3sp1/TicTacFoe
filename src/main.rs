use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;
use tic_tac_foe::app::{App};
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

        if let Event::Key(key) = event::read()? {
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

        if app.should_quit {
            return Ok(());
        }
    }
}
