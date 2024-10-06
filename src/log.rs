use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

pub struct Logger {
    file: File,
}

impl Logger {
    // Initialize the logger with a file path
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Logger> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)?;
        Ok(Logger { file })
    }

    // Write a log message to the file
    pub fn log(&mut self, message: &str) -> io::Result<()> {
        writeln!(self.file, "{}", message)?;
        self.file.flush()?;
        Ok(())
    }
}
