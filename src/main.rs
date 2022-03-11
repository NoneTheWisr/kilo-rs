use std::io::{self, Read};
use termion::raw::IntoRawMode;

type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
    IoError(io::Error),
    KiloError(String),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

fn main() -> Result<()> {
    let _stdout = io::stdout().into_raw_mode().unwrap();

    loop {
        let mut b: u8 = 0;
        io::stdin().read(std::slice::from_mut(&mut b))?;

        let b = b;
        let c = b as char;
        if c.is_control() {
            print!("{:?}\r\n", b);
        } else {
            print!("{:?} ({})\r\n", b, c);
        }
        if c == 'q' {
            break;
        }
    }
    
    Ok(())
}
