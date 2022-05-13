use std::env;

use anyhow::Result;

use kilo_rs::{app::App, runner::AppRunner};

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() > 2 {
        println!("USAGE: kilo [path_to_file]");
        return Ok(());
    }

    AppRunner::new(App::new(args)?).run()
}
