//! Plain text file reader
//!
//! Reads .txt files line by line with streaming (memory efficient).

use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
};

pub struct TextReader {
    lines: Lines<BufReader<File>>,
}
