use anyhow::Result;

use kilo_rs::app::App;
use rustea::crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType::All},
};

fn main() -> Result<()> {
    // let mut kilo = App::new()?;

    // let args: Vec<_> = env::args().skip(1).collect();
    // match args.len() {
    //     0 => {}
    //     1 => kilo.open_file(&args[0])?,
    //     _ => println!("USAGE: kilo [path_to_file]"),
    // }

    // kilo.run()
    let mut stdout = std::io::stdout();

    execute!(stdout, Clear(All))?;
    rustea::run(App::new().unwrap())?;
    execute!(stdout, Clear(All), MoveTo(0, 0))?;

    Ok(())
}
