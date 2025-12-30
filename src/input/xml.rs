//! Wikipedia XML dump parser
//!
//! Streaming parser for MediaWiki XML dumps.
//! Extracts article text and strips wikitext markup.

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

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

impl WikiXmlReader {
    /// Opens a Wikipedia XML dump file.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::open_with_options(path, true)
    }

    /// Opens a Wikipedia XML dump with custom options.
    pub fn open_with_options<P: AsRef<Path>>(
        path: P,
        main_namespace_only: bool,
    ) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::with_capacity(DEFAULT_BUF_SIZE, file);
        Ok(Self {
            reader,
            state: State::Idle,
            line_buffer: String::new(),
            current_title: String::new(),
            current_namespace: 0,
            current_text: String::new(),
            main_namespace_only,
        })
    }

    /// Parses the next article from the dump.
    fn parse_next_article(&mut self) -> io::Result<Option<WikiArticle>> {
        loop {
            self.line_buffer.clear();
            let bytes_read = self.reader.read_line(&mut self.line_buffer)?;

            // EOF
            if bytes_read == 0 {
                return Ok(None);
            }

            let line = self.line_buffer.trim();

            match self.state {
                State::Idle => {
                    if line.contains("<page>") {
                        self.state = State::InPage;
                        self.current_title.clear();
                        self.current_namespace = 0;
                        self.current_text.clear();
                    }
                }
                State::InPage => {
                    if line.contains("<title>") {
                        self.state = State::InTitle;
                        if let Some(content) = extract_single_line_tag_content(line, "title") {
                            self.current_title = content;
                            self.state = State::InPage;
                        }
                    } else if line.contains("<ns>") {
                        if let Some(content) = extract_single_line_tag_content(line, "ns") {
                            self.current_namespace = content.parse().unwrap_or(0);
                        }
                    } else if line.contains("<text>") {
                        self.state = State::InText;
                        // Handle text on same line as opening tag
                        if let Some(start) = line.find('>') {
                            let content = &line[start + 1..];
                            if let Some(end) = content.find("</text>") {
                                // Complete text on one line
                                self.current_text = content[..end].to_string();
                                self.state = State::InPage;
                            } else {
                                self.current_text = content.to_string();
                            }
                        }
                    } else if line.contains("</page>") {
                        self.state = State::Idle;

                        // Filter by namespace if requested
                        if self.main_namespace_only && self.current_namespace != 0 {
                            continue;
                        }

                        return Ok(Some(WikiArticle {
                            title: self.current_title.clone(),
                            namespace: self.current_namespace,
                            text: strip_wikitext(&self.current_text),
                        }));
                    }
                }
                State::InTitle => {
                    if line.contains("</title>") {
                        if let Some(end) = line.find("</title>") {
                            self.current_title.push_str(&line[..end]);
                        }
                        self.state = State::InPage;
                    } else {
                        self.current_title.push_str(line);
                    }
                }
                State::InNamespace => {
                    // Handled inline
                    self.state = State::InPage;
                }
                State::InText => {
                    if line.contains("</text>") {
                        if let Some(end) = line.find("</text>") {
                            self.current_text.push_str(&line[..end]);
                        }
                        self.state = State::InPage;
                    } else {
                        self.current_text.push('\n');
                        self.current_text.push_str(line);
                    }
                }
            }
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_comments() {
        assert_eq!(
            strip_wikitext("hello <!-- comment --> world"),
            "hello  world"
        );
    }

    #[test]
    fn test_strip_templates() {
        assert_eq!(strip_wikitext("hello {{template}} world"), "hello  world");
        assert_eq!(strip_wikitext("{{nested {{inner}}}}"), "");
    }

    #[test]
    fn test_strip_links() {
        assert_eq!(strip_wikitext("[[Link]]"), "Link");
        assert_eq!(strip_wikitext("[[Target|Display]]"), "Display");
        assert_eq!(strip_wikitext("[[Category:Test]]"), "");
        assert_eq!(strip_wikitext("[[File:Image.png]]"), "");
    }

    #[test]
    fn test_strip_formatting() {
        assert_eq!(strip_wikitext("'''bold'''"), "bold");
        assert_eq!(strip_wikitext("''italic''"), "italic");
    }

    #[test]
    fn test_strip_headings() {
        assert_eq!(strip_wikitext("== Heading =="), "Heading ");
        assert_eq!(strip_wikitext("=== Sub ==="), "Sub ");
    }

    #[test]
    fn test_strip_refs() {
        assert_eq!(strip_wikitext("text<ref>citation</ref>more"), "textmore");
        assert_eq!(strip_wikitext("text<ref name=\"x\"/>more"), "textmore");
    }

    #[test]
    fn test_strip_tables() {
        assert_eq!(
            strip_wikitext("before {| table content |} after"),
            "before  after"
        );
    }

    #[test]
    fn test_unicode() {
        assert_eq!(strip_wikitext("你好 [[世界|地球]] 再见"), "你好 地球 再见");
        assert_eq!(strip_wikitext("{{模板}} 中文"), " 中文");
    }
}
