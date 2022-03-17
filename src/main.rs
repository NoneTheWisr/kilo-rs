use std::io::{self, Read};
use std::slice;
use termion::raw::IntoRawMode;

fn read_byte(reader: &mut impl Read) -> io::Result<u8> {
    let mut byte: u8 = 0;
    reader.read(slice::from_mut(&mut byte)).map(|_| byte)
}

fn main() {
    let _stdout = io::stdout().into_raw_mode().unwrap();
    let mut stdin = io::stdin();

    loop {
        let b = read_byte(&mut stdin).unwrap();
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
}
