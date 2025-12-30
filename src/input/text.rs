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
