use std::env;

use anyhow::Result;

use kilo_rs::app::App;

fn main() -> Result<()> {
    let mut kilo = App::new()?;

    let args: Vec<_> = env::args().skip(1).collect();
    match args.len() {
        0 => {}
        1 => kilo.open_file(&args[0])?,
        _ => println!("USAGE: kilo [path_to_file]"),
    }

    kilo.run()
}