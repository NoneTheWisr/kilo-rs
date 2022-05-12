use std::env;

use anyhow::Result;

use kilo_rs::{app::App, term_utils::RawModeOverride};
use rustea::crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType::All},
};

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() > 2 {
        println!("USAGE: kilo [path_to_file]");
        return Ok(());
    }

    let _override = RawModeOverride::new();
    let mut stdout = std::io::stdout();

    execute!(stdout, Clear(All))?;
    rustea::run(App::new(args).unwrap())?;
    execute!(stdout, Clear(All), MoveTo(0, 0))?;

    Ok(())
}
