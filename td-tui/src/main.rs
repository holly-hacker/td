mod ui;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, path::Path};
use td_lib::database::{Database, DatabaseInfo};
use tui::{backend::CrosstermBackend, Terminal};
use ui::App;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        let name = std::env::args()
            .next()
            .expect("There should always be 1 item");
        println!("Usage: {name} <database.json>");
        return;
    }

    let path = Path::new(&args[0]);

    let database: Result<Database, _> = if !path.exists() {
        println!("The given database file ({path:?}) does not exist, creating a new one.");

        let db_info = DatabaseInfo::default();
        db_info.write(path).and_then(|_| db_info.try_into())
    } else {
        DatabaseInfo::read(path).and_then(|info| info.try_into())
    };

    match database {
        Ok(database) => {
            let err = run_app(database);
            if let Err(err) = err {
                println!("Error while running app: {err}");
            }
        }
        Err(err) => {
            println!("Error while reading database: {err}");
        }
    }
}

fn run_app(database: Database) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();

    // NOTE: could enable mouse capture here
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App { database };
    app.run(&mut terminal)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
