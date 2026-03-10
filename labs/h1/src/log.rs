use std::{
    fs::File,
    io::{self, Write},
};

pub struct Logger(pub File);

impl Logger {
    pub fn new(path: &str) -> Result<Self, io::Error> {
        let file = File::create(path)?;
        Ok(Self(file))
    }
}

impl Write for Logger {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

#[macro_export]
macro_rules! log_to {
    ($logger:expr, $($arg:tt)*) => {
    if true {
        let now = chrono::Local::now();
        let timestamp = now.format("[%H:%M:%S%.3f]");
        write!($logger, "{} {}\n", timestamp, format!($($arg)*)).expect("Failed to write");
        }
    };
}
