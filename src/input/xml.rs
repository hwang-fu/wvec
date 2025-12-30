//! Wikipedia XML dump parser
//!
//! Streaming parser for MediaWiki XML dumps.
//! Extracts article text and strips wikitext markup.

use std::{fs::File, io::BufReader};

/// Default buffer size for reading (24 KB)
const DEFAULT_BUF_SIZE: usize = 24 * 1024;

/// Parser state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Outside any relevant tag
    Idle,

    /// Inside <page> element
    InPage,

    /// Inside <title> element (within page)
    InTitle,

    /// Inside <ns> element (within page)
    InNamespace,

    /// Inside <text> element (within page)
    InText,
}

/// A single Wikipedia article extracted from the dump
#[derive(Debug, Clone)]
pub struct WikiArticle {
    /// Article title
    pub title: String,
    /// Article namespace (0 = main articles)
    pub namespace: i32,
    /// Article text content (wikitext stripped)
    pub text: String,
}

/// Streaming parser for Wikipedia XML dumps.
///
/// Yields `WikiArticle` items as it parses through the dump.
/// Memory-efficient: only holds one article at a time.
pub struct WikiXmlReader {
    /// Buffered reader for the XML file
    reader: BufReader<File>,

    /// Current parser state
    state: State,

    /// Buffer for reading lines
    line_buffer: String,

    /// Current article being built
    current_title: String,
    current_namespace: i32,
    current_text: String,

    /// Whether to filter to main namespace only (ns=0)
    main_namespace_only: bool,
}
