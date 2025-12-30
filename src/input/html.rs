//! HTML content stripper
//!
//! Strips HTML tags and decodes entities to extract plain text.

use std::{
    borrow::Cow,
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

/// Default buffer size for reading (24 KB)
const DEFAULT_BUF_SIZE: usize = 24 * 1024;

/// A streaming reader for HTML files that strips tags and extracts text.
pub struct HtmlReader {
    /// Buffered reader for the HTML file
    reader: BufReader<File>,
}

impl HtmlReader {
    /// Opens an HTML file for text extraction.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::with_capacity(DEFAULT_BUF_SIZE, file);
        Ok(Self { reader })
    }

    /// Reads and processes the entire HTML file, returning stripped text.
    pub fn read_all(&mut self) -> io::Result<String> {
        let mut content = String::new();
        self.reader.read_to_string(&mut content)?;
        Ok(strip_html(&content))
    }
}

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
    let mut skip_tag = String::new(); // Pre-allocated, reused across iterations

    while i < len {
        let rest = &html[i..];

        // When inside <script>, <style>, etc., skip everything until closing tag
        if skipping_content {
            // Case-insensitive comparison without allocation
            if rest
                .get(..skip_tag.len())
                .is_some_and(|s| s.eq_ignore_ascii_case(&skip_tag))
            {
                i += skip_tag.len();
                skipping_content = false;
                skip_tag.clear();
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
                    skip_tag.clear();
                    skip_tag.push_str("</");
                    skip_tag.push_str(&tag_name);
                    skip_tag.push('>');
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_basic_tags() {
        assert_eq!(strip_html("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html("<div>World</div>"), "World");
        assert_eq!(strip_html("<span>Text</span>"), "Text");
    }

    #[test]
    fn test_block_level_tags_add_space() {
        assert_eq!(strip_html("<p>Hello</p><p>World</p>"), "Hello World");
        assert_eq!(strip_html("<div>A</div><div>B</div>"), "A B");
        assert_eq!(strip_html("<h1>Title</h1><p>Content</p>"), "Title Content");
    }

    #[test]
    fn test_inline_tags_no_space() {
        assert_eq!(strip_html("<b>bold</b>"), "bold");
        assert_eq!(strip_html("Hello <b>World</b>"), "Hello World");
        assert_eq!(strip_html("<span>A</span><span>B</span>"), "AB");
    }

    #[test]
    fn test_nested_tags() {
        assert_eq!(
            strip_html("<div><p>Hello <b>World</b></p></div>"),
            "Hello World"
        );
        assert_eq!(
            strip_html("<article><section><p>Text</p></section></article>"),
            "Text"
        );
    }

    #[test]
    fn test_script_tag_removed() {
        assert_eq!(
            strip_html("<p>Hi</p><script>alert('bad')</script><p>Bye</p>"),
            "Hi Bye"
        );
        assert_eq!(
            strip_html("<script type='text/javascript'>var x = 1;</script>Hello"),
            "Hello"
        );
    }

    #[test]
    fn test_style_tag_removed() {
        assert_eq!(
            strip_html("<style>.class { color: red; }</style><p>Text</p>"),
            "Text"
        );
        assert_eq!(
            strip_html("<p>Before</p><style>body{}</style><p>After</p>"),
            "Before After"
        );
    }

    #[test]
    fn test_noscript_tag_removed() {
        assert_eq!(
            strip_html("<noscript>Enable JS</noscript><p>Content</p>"),
            "Content"
        );
    }

    #[test]
    fn test_svg_tag_removed() {
        assert_eq!(
            strip_html("<svg><circle cx='50' cy='50' r='40'/></svg><p>Text</p>"),
            "Text"
        );
    }

    #[test]
    fn test_self_closing_script() {
        assert_eq!(strip_html("<script /><p>Hello</p>"), "Hello");
        assert_eq!(strip_html("<script/><p>World</p>"), "World");
    }

    #[test]
    fn test_case_insensitive_tags() {
        assert_eq!(strip_html("<P>Hello</P>"), "Hello");
        assert_eq!(strip_html("<SCRIPT>bad</SCRIPT>Text"), "Text");
        assert_eq!(strip_html("<Script>bad</script>Text"), "Text");
        assert_eq!(strip_html("<DIV>A</DIV><div>B</div>"), "A B");
    }

    #[test]
    fn test_html_comments() {
        assert_eq!(strip_html("Hello <!-- comment --> World"), "Hello World");
        assert_eq!(strip_html("<!-- comment -->Text"), "Text");
        assert_eq!(strip_html("Text<!-- comment -->"), "Text");
        assert_eq!(strip_html("A<!-- multi\nline\ncomment -->B"), "AB");
    }

    #[test]
    fn test_tag_with_attributes() {
        assert_eq!(
            strip_html("<div class='container' id='main'>Content</div>"),
            "Content"
        );
        assert_eq!(strip_html("<a href='http://example.com'>Link</a>"), "Link");
        assert_eq!(strip_html("<img src='image.png' alt='description' />"), "");
    }

    #[test]
    fn test_br_and_hr_tags() {
        assert_eq!(strip_html("Line1<br>Line2"), "Line1 Line2");
        assert_eq!(strip_html("Line1<br/>Line2"), "Line1 Line2");
        assert_eq!(strip_html("Above<hr>Below"), "Above Below");
    }

    #[test]
    fn test_table_tags() {
        assert_eq!(
            strip_html("<table><tr><td>A</td><td>B</td></tr></table>"),
            "A B"
        );
    }

    #[test]
    fn test_list_tags() {
        assert_eq!(strip_html("<ul><li>One</li><li>Two</li></ul>"), "One Two");
        assert_eq!(
            strip_html("<ol><li>First</li><li>Second</li></ol>"),
            "First Second"
        );
    }

    #[test]
    fn test_named_entities() {
        assert_eq!(strip_html("&amp;"), "&");
        assert_eq!(strip_html("&lt;"), "<");
        assert_eq!(strip_html("&gt;"), ">");
        assert_eq!(strip_html("&quot;"), "\"");
        assert_eq!(strip_html("&apos;"), "'");
        assert_eq!(strip_html("&nbsp;"), "");
    }

    #[test]
    fn test_named_entities_in_context() {
        assert_eq!(strip_html("A &amp; B"), "A & B");
        assert_eq!(strip_html("&lt;div&gt;"), "<div>");
        assert_eq!(strip_html("&copy; 2024"), "© 2024");
    }

    #[test]
    fn test_numeric_entities_decimal() {
        assert_eq!(strip_html("&#60;"), "<");
        assert_eq!(strip_html("&#62;"), ">");
        assert_eq!(strip_html("&#38;"), "&");
        assert_eq!(strip_html("&#20320;&#22909;"), "你好");
    }

    #[test]
    fn test_numeric_entities_hex() {
        assert_eq!(strip_html("&#x3C;"), "<");
        assert_eq!(strip_html("&#x3E;"), ">");
        assert_eq!(strip_html("&#x26;"), "&");
        assert_eq!(strip_html("&#x4F60;&#x597D;"), "你好");
        // Uppercase X
        assert_eq!(strip_html("&#X3C;"), "<");
    }

    #[test]
    fn test_unknown_entity_kept() {
        assert_eq!(strip_html("&unknown;"), "&unknown;");
        assert_eq!(strip_html("&fake;"), "&fake;");
    }

    #[test]
    fn test_incomplete_entity() {
        assert_eq!(strip_html("&amp no semicolon"), "&amp no semicolon");
        assert_eq!(strip_html("AT&T"), "AT&T");
    }

    #[test]
    fn test_entity_too_long() {
        assert_eq!(strip_html("&verylongentity;"), "&verylongentity;");
    }

    #[test]
    fn test_unicode_content() {
        assert_eq!(strip_html("<p>你好世界</p>"), "你好世界");
        assert_eq!(strip_html("<p>こんにちは</p>"), "こんにちは");
        assert_eq!(strip_html("<p>🎉🎊🎁</p>"), "🎉🎊🎁");
    }

    #[test]
    fn test_unicode_in_script() {
        assert_eq!(
            strip_html("<script>var x = '中文';</script><p>Text</p>"),
            "Text"
        );
    }

    #[test]
    fn test_mixed_unicode_and_entities() {
        assert_eq!(strip_html("你好 &amp; 世界"), "你好 & 世界");
        assert_eq!(strip_html("&#20320;好"), "你好");
    }

    #[test]
    fn test_normalize_multiple_spaces() {
        assert_eq!(strip_html("Hello    World"), "Hello World");
        assert_eq!(strip_html("A  B   C    D"), "A B C D");
    }

    #[test]
    fn test_normalize_newlines() {
        assert_eq!(strip_html("Hello\nWorld"), "Hello World");
        assert_eq!(strip_html("A\n\n\nB"), "A B");
    }

    #[test]
    fn test_normalize_tabs() {
        assert_eq!(strip_html("Hello\tWorld"), "Hello World");
        assert_eq!(strip_html("A\t\t\tB"), "A B");
    }

    #[test]
    fn test_normalize_mixed_whitespace() {
        assert_eq!(strip_html("A \n\t B"), "A B");
        assert_eq!(strip_html("  \n\t  Hello  \n\t  "), "Hello");
    }

    #[test]
    fn test_trim_leading_trailing() {
        assert_eq!(strip_html("   Hello   "), "Hello");
        assert_eq!(strip_html("\n\nHello\n\n"), "Hello");
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(strip_html(""), "");
    }

    #[test]
    fn test_plain_text_no_html() {
        assert_eq!(strip_html("Just plain text"), "Just plain text");
    }

    #[test]
    fn test_only_tags() {
        assert_eq!(strip_html("<div><span></span></div>"), "");
    }

    #[test]
    fn test_unclosed_tag() {
        // Unclosed tag at end - '<' without '>' is kept as text
        assert_eq!(strip_html("Hello <unclosed"), "Hello <unclosed");
    }

    #[test]
    fn test_angle_bracket_in_text() {
        // Lone < or > not part of tag
        assert_eq!(strip_html("5 > 3"), "5 > 3");
        assert_eq!(strip_html("a<b means less"), "a<b means less");
    }

    #[test]
    fn test_empty_tag() {
        assert_eq!(strip_html("<>Content</>"), "Content");
    }

    #[test]
    fn test_deeply_nested() {
        assert_eq!(
            strip_html("<div><div><div><div><p>Deep</p></div></div></div></div>"),
            "Deep"
        );
    }

    #[test]
    fn test_decode_entity_named() {
        let (decoded, consumed) = decode_entity("&amp;rest").unwrap();
        assert_eq!(decoded.as_ref(), "&");
        assert_eq!(consumed, 5);
    }

    #[test]
    fn test_decode_entity_decimal() {
        let (decoded, consumed) = decode_entity("&#60;rest").unwrap();
        assert_eq!(decoded.as_ref(), "<");
        assert_eq!(consumed, 5);
    }

    #[test]
    fn test_decode_entity_hex() {
        let (decoded, consumed) = decode_entity("&#x3C;rest").unwrap();
        assert_eq!(decoded.as_ref(), "<");
        assert_eq!(consumed, 6);
    }

    #[test]
    fn test_decode_entity_unicode() {
        let (decoded, consumed) = decode_entity("&#20320;rest").unwrap();
        assert_eq!(decoded.as_ref(), "你");
        assert_eq!(consumed, 8);
    }

    #[test]
    fn test_decode_entity_invalid() {
        assert!(decode_entity("&unknown;").is_none());
        assert!(decode_entity("no entity").is_none());
        assert!(decode_entity("&nosemicolon").is_none());
    }

    #[test]
    fn test_normalize_whitespace_basic() {
        assert_eq!(normalize_whitespace("a  b"), "a b");
        assert_eq!(normalize_whitespace("  a  b  "), "a b");
    }

    #[test]
    fn test_normalize_whitespace_empty() {
        assert_eq!(normalize_whitespace(""), "");
        assert_eq!(normalize_whitespace("   "), "");
    }

    #[test]
    fn test_normalize_whitespace_no_change() {
        assert_eq!(normalize_whitespace("hello world"), "hello world");
    }

    #[test]
    fn test_html_reader() {
        let dir = std::env::temp_dir();
        let path = dir.join("wvec_test_html.html");

        {
            let mut file = File::create(&path).unwrap();
            writeln!(
                file,
                "<html><body><h1>Title</h1><p>Hello &amp; World</p></body></html>"
            )
            .unwrap();
        }

        let mut reader = HtmlReader::open(&path).unwrap();
        let text = reader.read_all().unwrap();

        assert_eq!(text, "Title Hello & World");

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_html_reader_with_script() {
        let dir = std::env::temp_dir();
        let path = dir.join("wvec_test_html_script.html");

        {
            let mut file = File::create(&path).unwrap();
            writeln!(
                file,
                "<html><script>var x = 1;</script><body><p>Content</p></body></html>"
            )
            .unwrap();
        }

        let mut reader = HtmlReader::open(&path).unwrap();
        let text = reader.read_all().unwrap();

        assert_eq!(text, "Content");

        std::fs::remove_file(&path).unwrap();
    }
}
