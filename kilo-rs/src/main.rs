use std::env;

use anyhow::Result;

use kilo_rs::{
    app::{App, StartupArgs},
    runner::AppRunner,
};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let startup_args = match args.len() {
        0 => Default::default(),
        1 => StartupArgs {
            file: Some(args.next().unwrap()),
        },
        _ => {
            println!("USAGE: kilo [path_to_file]");
            return Ok(());
        }
    };

    AppRunner::new(App::new(startup_args)?).run()
}
