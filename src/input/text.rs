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

    /// Reads next line, truncating if it exceeds max_line_length.
    /// Returns None at EOF.
    fn read_next_line(&mut self) -> io::Result<Option<String>> {
        self.buffer.clear();
        let mut total_read = 0;

        loop {
            let available = self.reader.fill_buf()?;

            // EOF: return remaining buffer content or None
            if available.is_empty() {
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(std::mem::take(&mut self.buffer)))
                };
            }

            // Search for newline in the available buffer
            let newline_pos = available.iter().position(|&b| b == b'\n');

            // Determine how many bytes to read from this chunk:
            // - Up to newline if found, otherwise entire chunk
            // - But never exceed remaining capacity
            let chunk_end = newline_pos.unwrap_or(available.len());
            let remaining_capacity = self.max_line_length.saturating_sub(total_read);
            let to_take = chunk_end.min(remaining_capacity);

            // Append bytes to buffer, handling UTF-8 validity
            if to_take > 0 {
                match std::str::from_utf8(&available[..to_take]) {
                    Ok(s) => {
                        self.buffer.push_str(s);
                    }
                    Err(e) => {
                        // Partial UTF-8: take only the valid portion
                        let valid_up_to = e.valid_up_to();
                        if valid_up_to > 0 {
                            // SAFETY: we just verified these bytes are valid UTF-8
                            let s =
                                unsafe { std::str::from_utf8_unchecked(&available[..valid_up_to]) };
                            self.buffer.push_str(s);
                        }
                    }
                }
            }

            total_read += to_take;

            // Found newline: consume it and return the complete line
            if let Some(pos) = newline_pos {
                // Consume bytes up to and including the newline
                self.reader.consume(pos + 1);

                // Handle Windows-style line endings (\r\n)
                if self.buffer.ends_with('\r') {
                    self.buffer.pop();
                }

                return Ok(Some(std::mem::take(&mut self.buffer)));
            }

            // Consume the bytes we processed
            self.reader.consume(chunk_end);

            // Hit max capacity: skip remaining bytes until newline
            if total_read >= self.max_line_length {
                self.skip_until_newline()?;
                return Ok(Some(std::mem::take(&mut self.buffer)));
            }
        }
    }

    /// Discards bytes until the next newline character.
    fn skip_until_newline(&mut self) -> io::Result<()> {
        loop {
            let available = self.reader.fill_buf()?;

            // EOF: nothing more to skip
            if available.is_empty() {
                return Ok(());
            }

            // Look for newline in current buffer
            match available.iter().position(|&b| b == b'\n') {
                Some(pos) => {
                    // Found newline: consume up to (inclusive) it, done
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

    /// Advances the iterator and returns the next line.
    fn next(&mut self) -> Option<Self::Item> {
        match self.read_next_line() {
            Ok(Some(line)) => Some(Ok(line)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
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

    #[test]
    fn test_long_line_truncation() {
        let dir = std::env::temp_dir();
        let path = dir.join("wvec_test_with_long_line.txt");

        // Create test file with a 1000-char line
        {
            let mut file = File::create(&path).unwrap();
            let long_line = "x".repeat(1000);
            writeln!(file, "{}", long_line).unwrap();
            writeln!(file, "a fairly short line").unwrap();
        }

        // Reader with 100-char limit
        let reader = TextReader::open_with_limit(&path, 100).unwrap();
        let lines: Vec<String> = reader.map(|l| l.unwrap()).collect();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), 100); // Truncated to limit
        assert_eq!(lines[1], "a fairly short line"); // Unaffected

        std::fs::remove_file(&path).unwrap();
    }
}
