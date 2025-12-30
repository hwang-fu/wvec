//! Plain text file reader
//!
//! Reads .txt files line by line with streaming (memory efficient).

use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines},
    path::Path,
};

/// A stream reader for plain text files.
pub struct TextReader {
    lines: Lines<BufReader<File>>,
}

impl TextReader {
    /// Opens a text file for streaming line-by-line reading.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            lines: reader.lines(),
        })
    }
}

impl Iterator for TextReader {
    type Item = io::Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next()
    }
}
