//! HTML content stripper
//!
//! Strips HTML tags and decodes entities to extract plain text.

/// Default buffer size for reading (24 KB)
const DEFAULT_BUF_SIZE: usize = 3 * 1024;
