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

impl WikiXmlReader {}

/// Extracts content between simple single line opening and closing tags on a single line.
/// e.g., <title>Article Name</title>, <ns>0</ns>
/// For <text>...content...</text> which spans many lines we use state machine instead.
/// Returns None if tags aren't found or content spans multiple lines.
fn extract_single_line_tag_content(line: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);

    let start = line.find(&open_tag)?;
    let end = line.find(&close_tag)?;

    let content_start = start + open_tag.len();
    if content_start < end {
        Some(line[content_start..end].to_string())
    } else {
        None
    }
}

/// Strips wikitext markup from article text.
///
/// Removes:
/// - `[[Link]]` and `[[Link|Display]]` -> keeps display text or link
///
/// - `{{Template}}`        -> removed entirely
/// - `{| table |}`         -> removed
/// - `<!-- comments -->`   -> removed
/// - `<ref>...</ref>`      -> removed
///
/// - `'''bold'''` and `''italic''` -> keeps text
/// - `== Headings ==`              -> keeps text
fn strip_wikitext(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let len = text.len();
    let mut i = 0;

    while i < len {
        let rest = &text[i..];

        // Skip HTML comments <!-- -->
        if rest.starts_with("<!--")
            && let Some(end) = rest.find("-->")
        {
            i += (end + 3);
            continue;
        }

        // Skip <ref>...</ref> tags
        if rest.starts_with("<ref") {
            if let Some(end) = rest.find("</ref>") {
                i += (end + 6);
                continue;
            } else if let Some(end) = rest.find("/>") {
                i += (end + 2);
                continue;
            }
            // Skip "<ref" itself when couldn't find the end tag.
            i += 4;
            continue;
        }

        // Skip templates {{ }}
        if rest.starts_with("{{") {
            let mut depth = 2;
            let mut j = 2;
            let rest_bytes = rest.as_bytes();

            while j < rest.len() && depth > 0 {
                if rest[j..].starts_with("{{") {
                    depth += 2;
                    j += 2;
                } else if rest[j..].starts_with("}}") {
                    depth -= 2;
                    j += 2;
                } else {
                    // Advance one UTF-8 character
                    j += 1;
                    while j < rest.len() && (rest_bytes[j] & 0xC0) == 0x80 {
                        j += 1;
                    }
                }
            }

            i += j;
            continue;
        }

        // Skip tables {| |}
        if let Some(end) = rest.starts_with("{|").then(|| rest.find("|}")).flatten() {
            i += end + 2;
            continue;
        }

        // Handle links [[ ]]
        if rest.starts_with("[[") {
            let mut j = 2;
            let mut depth = 1;
            let rest_bytes = rest.as_bytes();
            while j < rest.len() && depth > 0 {
                if rest[j..].starts_with("[[") {
                    depth += 1;
                    j += 2;
                } else if rest[j..].starts_with("]]") {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    j += 2;
                } else {
                    // Advance one UTF-8 character
                    j += 1;
                    while j < rest.len() && (rest_bytes[j] & 0xC0) == 0x80 {
                        j += 1;
                    }
                }
            }

            // Extract link content
            let link_content = &rest[2..j];

            // Use display text if present (after |), otherwise use link target
            let display = if let Some(pipe_pos) = link_content.rfind('|') {
                &link_content[pipe_pos + 1..]
            } else {
                // Skip category/file links
                if link_content.starts_with("Category:")
                    || link_content.starts_with("File:")
                    || link_content.starts_with("Image:")
                {
                    i += j + 2;
                    continue;
                }
                link_content
            };

            result.push_str(display);
            i += (j + 2);
            continue;
        }

        // Skip bold/italic markers
        if rest.starts_with("''") {
            let mut count = 0;
            for ch in rest.chars() {
                if ch == '\'' {
                    count += 1;
                } else {
                    break;
                }
            }
            i += count;
            continue;
        }

        // Skip heading markers = at start of line
        let at_line_start = (i == 0) || (text.as_bytes()[i - 1] == b'\n');
        if rest.starts_with('=') && at_line_start {
            let mut j = 0;
            for ch in rest.chars() {
                if ch == '=' {
                    j += 1;
                } else {
                    break;
                }
            }

            // Skip leading space after =
            if rest[j..].starts_with(' ') {
                j += 1;
            }
            i += j;
            continue;
        }

        // Remove trailing = from headings
        if rest.starts_with('=') {
            let next_char = rest.chars().nth(1);
            if next_char == Some('\n') || next_char == Some('=') {
                let mut j = 0;
                for ch in rest.chars() {
                    if ch == '=' {
                        j += 1;
                    } else {
                        break;
                    }
                }
                i += j;
                continue;
            }
        }

        // Push current character and advance
        let ch = rest.chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }

    result.trim().to_string()
}
