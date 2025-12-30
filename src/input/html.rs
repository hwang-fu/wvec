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
