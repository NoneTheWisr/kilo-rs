use std::io::{self, Read};
use std::os::unix::io::{RawFd, AsRawFd};
use std::slice;

use termios::Termios;

pub struct TermiosOverride {
    fd: RawFd,
    original_termios: Termios,
}

impl TermiosOverride {
    pub fn new() -> Self {
        let fd = io::stdin().as_raw_fd();
        let original_termios = Termios::from_fd(fd).unwrap();

        let r#override = Self::r#override(original_termios);
        termios::tcsetattr(fd, termios::TCSAFLUSH, &r#override).unwrap();

        TermiosOverride { fd, original_termios }
    }

    fn r#override(mut termios: Termios) -> Termios {
        use termios::*;

        termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        termios.c_oflag &= !(OPOST);
        termios.c_cflag |= CS8;
        termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        termios.c_cc[VMIN] = 0;
        termios.c_cc[VTIME] = 1;

        termios
    }
}

impl Drop for TermiosOverride {
    fn drop(&mut self) {
        termios::tcsetattr(self.fd, termios::TCSAFLUSH, &self.original_termios).unwrap();
    }
}

fn read_byte(reader: &mut impl Read) -> io::Result<u8> {
    let mut byte: u8 = 0;
    reader.read(slice::from_mut(&mut byte)).map(|_| byte)
}

struct Editor; 

impl Editor {
    fn new() -> Self {
        Self
    }

    fn run(&self) {
        let _termios_override = TermiosOverride::new();
        let mut stdin = io::stdin();

        loop {
            let b = read_byte(&mut stdin).unwrap();
            let c = b as char;

            if c.is_control() {
                print!("{b}\r\n");
            } else {
                print!("{b} ('{c}')\r\n")
            }
            
            if c == 'q' {
                break;
            }
        }
    }
}

fn main() {
    Editor::new().run()
}
