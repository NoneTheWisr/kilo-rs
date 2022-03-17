use std::io::{self, Read};
use std::slice;
use std::time::Duration;
use termion::raw::IntoRawMode;

const TIMEOUT: Duration = Duration::from_millis(100);

struct PollingStdin {
    sources: popol::Sources<()>,
    events: popol::Events<()>,
    stdin: io::Stdin,
    timeout: Duration,
}

impl PollingStdin {
    fn new(timeout: Duration) -> Self {
        let mut sources = popol::Sources::with_capacity(1);
        let events = popol::Events::with_capacity(1);
        let stdin = io::stdin();

        sources.register((), &stdin, popol::interest::READ);

        Self { sources, events, stdin, timeout }
    }

    fn read_byte(&mut self, default: u8) -> io::Result<u8> {
        let mut byte: u8 = 0;
        match self.sources.wait_timeout(&mut self.events, self.timeout) {
            Ok(()) => self.stdin.read(slice::from_mut(&mut byte)).map(|_| byte),
            Err(err) if err.kind() == io::ErrorKind::TimedOut => Ok(default),
            Err(err) => Err(err),
        }
    }
}

fn main() {
    let _stdout = io::stdout().into_raw_mode().unwrap();
    let mut stdin = PollingStdin::new(TIMEOUT);

    loop {
        let b = stdin.read_byte(0).unwrap();
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
