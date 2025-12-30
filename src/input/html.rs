//! HTML content stripper
//!
//! Strips HTML tags and decodes entities to extract plain text.

use std::{fs::File, io::BufReader};

/// Default buffer size for reading (24 KB)
const DEFAULT_BUF_SIZE: usize = 3 * 1024;

/// A streaming reader for HTML files that strips tags and extracts text.
pub struct HtmlReader {
    /// Buffered reader for the HTML file
    reader: BufReader<File>,

    /// Buffer for accumulating text content
    buffer: String,

    /// Whether we're inside a tag that should be completely ignored
    in_ignored_tag: bool,

    /// The ignored tag name (script, style, etc.)
    ignored_tag: String,
}

impl HtmlReader {}

/// Normalizes whitespace: collapses multiple spaces/newlines into single spaces.
fn normalize_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_space = true; // Start true to trim leading space

    for ch in s.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(ch);
            last_was_space = false;
        }
    }

    // Trim trailing space
    if result.ends_with(' ') {
        result.pop();
    }

    result
}
