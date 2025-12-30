//! Wikipedia XML dump parser
//!
//! Streaming parser for MediaWiki XML dumps.
//! Extracts article text and strips wikitext markup.

/// Default buffer size for reading (8 KB)
const DEFAULT_BUF_SIZE: usize = 8 * 1024;
