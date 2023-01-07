mod keybinds;
mod ui;
mod utils;

use std::{error::Error, path::PathBuf};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};
use ui::AppState;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        let name = std::env::args()
            .next()
            .expect("There should always be 1 item");
        println!("Usage: {name} <database.json>");
        return;
    }

    let path = PathBuf::from(&args[0]);
    let app = AppState::create(path).unwrap();

    if let Err(e) = run_app(app) {
        println!("Error while running app: {e}");
    }
}

fn run_app(mut app: AppState) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();

    // NOTE: could enable mouse capture here
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    app.run_loop(&mut terminal)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
