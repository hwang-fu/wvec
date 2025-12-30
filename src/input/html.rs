//! HTML content stripper
//!
//! Strips HTML tags and decodes entities to extract plain text.

use std::{borrow::Cow, fs::File, io::BufReader, usize};

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
    let mut skipping_content = false;
    let mut skip_until_tag = String::new(); // Pre-allocated, reused across iterations

    while i < len {
        let rest = &html[i..];

        // When inside <script>, <style>, etc., skip everything until closing tag
        if skipping_content {
            // Case-insensitive comparison without allocation
            if rest.len() >= skip_until_tag.len()
                && rest[..skip_until_tag.len()].eq_ignore_ascii_case(&skip_until_tag)
            {
                i += skip_until_tag.len();
                skipping_content = false;
                skip_until_tag.clear();
                continue;
            }

            // Skip one complete UTF-8 character (not just one byte!)
            // This prevents panic when slicing multi-byte characters
            let ch = rest.chars().next().unwrap();
            i += ch.len_utf8();
            continue;
        }

        // Skip HTML comments
        if rest.starts_with("<!--")
            && let Some(end) = rest.find("-->")
        {
            i += end + 3; // Skip past "-->"
            continue;
        }

        // Handle HTML tags
        if rest.starts_with('<')
            && let Some(tag_end) = rest.find('>')
        {
            let tag_content = &rest[1..tag_end];

            // Extract tag name: take alphanumeric chars and lowercase them
            // e.g., "DIV class='foo'" -> "div"
            let tag_name: String = tag_content
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric())
                .map(|c| c.to_ascii_lowercase())
                .collect();

            // Check if this tag's content should be ignored entirely
            if matches!(tag_name.as_str(), "script" | "style" | "noscript" | "svg") {
                // Only enter ignored mode if not self-closing (e.g., <script />)
                if !tag_content.ends_with('/') {
                    skipping_content = true;

                    // Pre-build the closing tag to avoid repeated allocations
                    skip_until_tag.clear();
                    skip_until_tag.push_str("</");
                    skip_until_tag.push_str(&tag_name);
                    skip_until_tag.push('>');
                }
            }

            // Insert space for block-level tags to preserve word boundaries
            // e.g., "<p>Hello</p><p>World</p>" -> "Hello World" not "HelloWorld"
            if matches!(
                tag_name.as_str(),
                "p" | "div"
                    | "br"
                    | "li"
                    | "tr"
                    | "td"
                    | "th"
                    | "h1"
                    | "h2"
                    | "h3"
                    | "h4"
                    | "h5"
                    | "h6"
                    | "blockquote"
                    | "pre"
                    | "hr"
                    | "article"
                    | "section"
                    | "header"
                    | "footer"
            ) {
                result.push(' ');
            }

            i += tag_end + 1; // Skip past '>'
            continue;
        }

        // Decode HTML entities
        // e.g., "&amp;" -> "&",
        //       "&#60;" -> "<"
        // If decoding fails, fall through to treat '&' as regular char
        if rest.starts_with('&')
            && let Some((decoded, consumed)) = decode_entity(rest)
        {
            result.push_str(&decoded);
            i += consumed;
            continue;
        }

        // Regular character
        // Push the character and advance by its UTF-8 byte length
        let ch = rest.chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }

    normalize_whitespace(&result)
}

/// Decodes an HTML entity at the start of the string.
/// Returns (decoded_string, bytes_consumed) or None if not a valid entity.
fn decode_entity(s: &str) -> Option<(Cow<'static, str>, usize)> {
    let end = s.find(';')?;
    if end > 12 {
        return None; // Too long, probably not an entity
    }

    let entity = &s[..=end];
    let decoded: Cow<'static, str> = match entity {
        // Common named entities
        "&amp;" => "&".into(),
        "&lt;" => "<".into(),
        "&gt;" => ">".into(),
        "&quot;" => "\"".into(),
        "&apos;" => "'".into(),
        "&nbsp;" => " ".into(),
        "&copy;" => "©".into(),
        "&reg;" => "®".into(),
        "&trade;" => "™".into(),
        "&mdash;" => "—".into(),
        "&ndash;" => "–".into(),
        "&lsquo;" => "'".into(),
        "&rsquo;" => "'".into(),
        "&ldquo;" => "\"".into(),
        "&rdquo;" => "\"".into(),
        "&hellip;" => "…".into(),
        "&bull;" => "•".into(),
        "&euro;" => "€".into(),
        "&pound;" => "£".into(),
        "&yen;" => "¥".into(),
        "&cent;" => "¢".into(),
        _ => {
            // Try numeric entity
            if entity.starts_with("&#") {
                let num_str = &entity[2..entity.len() - 1];

                let code_point = if num_str.starts_with('x') || num_str.starts_with('X') {
                    u32::from_str_radix(&num_str[1..], 16).ok()?
                } else {
                    num_str.parse().ok()?
                };

                let ch = char::from_u32(code_point)?;
                return Some((ch.to_string().into(), entity.len()));
            }

            // Unknown entity, keep as-is
            return None;
        }
    };

    Some((decoded, entity.len()))
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
