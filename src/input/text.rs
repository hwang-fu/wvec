//! Plain text file reader
//!
//! Reads .txt files line by line with streaming (memory efficient).

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

/// Default maximum line length (10 MB)
pub const DEFAULT_MAX_LINE_LENGTH: usize = 10 * 1024 * 1024;

/// A stream reader for plain text files.
pub struct TextReader {
    /// Buffered reader wrapping the underlying file.
    /// Reads in 8KB chunks by default.
    reader: BufReader<File>,

    /// Maximum allowed line length in bytes.
    /// Lines exceeding this limit are truncated, and remaining bytes
    /// are discarded until the next newline.
    max_line_length: usize,

    /// Reusable buffer for building the current line.
    /// Cleared between lines to avoid repeated allocations.
    buffer: String,
}

impl TextReader {
    /// Opens a text file for streaming line-by-line reading.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::open_with_limit(path, DEFAULT_MAX_LINE_LENGTH)
    }

    /// Opens a text file with custom max line length.
    pub fn open_with_limit<P: AsRef<Path>>(path: P, max_line_length: usize) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            reader,
            max_line_length,
            buffer: String::new(),
        })
    }

    /// Discards bytes until the next newline character.
    fn skip_until_newline(&mut self) -> io::Result<()> {
        loop {
            let available = self.reader.fill_buf()?;

            if available.is_empty() {
                return Ok(());
            }

            match available.iter().position(|&b| b == b'\n') {
                Some(pos) => {
                    // Found newline: consume up to and including it, done
                    self.reader.consume(pos + 1);
                    return Ok(());
                }
                None => {
                    // No newline: discard entire buffer, continue
                    let len = available.len();
                    self.reader.consume(len);
                }
            }
        }
    }
}

impl Iterator for TextReader {
    type Item = io::Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_read_lines() {
        let dir = std::env::temp_dir();
        let path = dir.join("wvec_test_text.txt");

        {
            let mut file = File::create(&path).unwrap();
            writeln!(file, "hi").unwrap();
            writeln!(file, "it is just a text test file").unwrap();
            writeln!(file, "a line").unwrap();
            writeln!(file, "another line").unwrap();
        }

        let reader = TextReader::open(&path).unwrap();
        let lines: Vec<String> = reader.map(|l| l.unwrap()).collect();

        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "hi");
        assert_eq!(lines[1], "it is just a text test file");
        assert_eq!(lines[2], "a line");
        assert_eq!(lines[3], "another line");

        std::fs::remove_file(&path).unwrap();
    }
}
