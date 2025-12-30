//! HTML content stripper
//!
//! Strips HTML tags and decodes entities to extract plain text.

use std::{fs::File, io::BufReader, usize};

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

/// Strips HTML tags and decodes entities from text.
///
/// - Removes all HTML tags
/// - Removes content inside `<script>`, `<style>`, `<noscript>` tags
/// - Decodes common HTML entities
/// - Preserves text content
pub fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut i = 0;
    let len = html.len();

    // Track ignored tags (script, style, etc.)
    let mut in_ignored = false;
    let mut ignored_tag = String::new();

    while i < len {
        let rest = &html[i..];

        // Check for ignored tag endings first
        if in_ignored {
            let close_tag = format!("</{}>", ignored_tag);
            if rest.to_lowercase().starts_with(&close_tag) {
                i += close_tag.len();
                in_ignored = false;
                ignored_tag.clear();
                continue;
            }
            // Skip this character, we're inside ignored content
            i += 1;
            continue;
        }

        // Check for comment <!-- -->
        if rest.starts_with("<!--")
            && let Some(end) = rest.find("-->")
        {
            i += end + 3;
            continue;
        }

        // Check for tag start
        if rest.starts_with('<') {
            // Find tag end
            if let Some(tag_end) = rest.find('>') {}
        }
    }

    result
}

/// Decodes an HTML entity at the start of the string.
/// Returns (decoded_string, bytes_consumed) or None if not a valid entity.
fn decode_entity(s: &str) -> Option<(String, usize)> {
    let end = s.find(';')?;
    if end > 12 {
        return None; // Too long, probably not an entity
    }

    let entity = &s[..=end];
    let decoded = match entity {
        // Common named entities
        "&amp;" => "&",
        "&lt;" => "<",
        "&gt;" => ">",
        "&quot;" => "\"",
        "&apos;" => "'",
        "&nbsp;" => " ",
        "&copy;" => "©",
        "&reg;" => "®",
        "&trade;" => "™",
        "&mdash;" => "—",
        "&ndash;" => "–",
        "&lsquo;" => "'",
        "&rsquo;" => "'",
        "&ldquo;" => "\"",
        "&rdquo;" => "\"",
        "&hellip;" => "…",
        "&bull;" => "•",
        "&euro;" => "€",
        "&pound;" => "£",
        "&yen;" => "¥",
        "&cent;" => "¢",
        _ => {
            // Try numeric entity
            if entity.starts_with("&#") {
                let num_str = &entity[2..entity.len() - 1];
                let code_point = if num_str.starts_with('x') || num_str.starts_with('X') {
                    // Hexadecimal
                    u32::from_str_radix(&num_str[1..], 16).ok()?
                } else {
                    // Decimal
                    num_str.parse().ok()?
                };
                let ch = char::from_u32(code_point)?;
                return Some((ch.to_string(), entity.len()));
            }
            // Unknown entity, keep as-is
            return None;
        }
    };

    Some((decoded.to_string(), entity.len()))
}

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
