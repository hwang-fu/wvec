//! Wikipedia XML dump parser
//!
//! Streaming parser for MediaWiki XML dumps.
//! Extracts article text and strips wikitext markup.

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
