//! Text normalization
//!
//! - Case folding (lowercase for Latin scripts)
//! - Punctuation normalization
//! - Whitespace normalization

/// Checks if a character is CJK ideograph (Han character)
pub fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}'   |   // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}'   |   // CJK Unified Ideographs Extension A
        '\u{20000}'..='\u{2A6DF}' |   // CJK Unified Ideographs Extension B
        '\u{2A700}'..='\u{2B73F}' |   // CJK Unified Ideographs Extension C
        '\u{2B740}'..='\u{2B81F}' |   // CJK Unified Ideographs Extension D
        '\u{2B820}'..='\u{2CEAF}' |   // CJK Unified Ideographs Extension E
        '\u{2CEB0}'..='\u{2EBEF}' |   // CJK Unified Ideographs Extension F
        '\u{30000}'..='\u{3134F}' |   // CJK Unified Ideographs Extension G
        '\u{31350}'..='\u{323AF}' |   // CJK Unified Ideographs Extension H
        '\u{2EBF0}'..='\u{2EE5F}' |   // CJK Unified Ideographs Extension I
        '\u{F900}'..='\u{FAFF}'   |   // CJK Compatibility Ideographs
        '\u{2F800}'..='\u{2FA1F}'     // CJK Compatibility Ideographs Supplement
    )
}

/// Checks if a character is Korean Hangul
pub fn is_hangul(ch: char) -> bool {
    matches!(ch,
        '\u{AC00}'..='\u{D7AF}' |  // Hangul Syllables
        '\u{1100}'..='\u{11FF}' |  // Hangul Jamo
        '\u{A960}'..='\u{A97F}' |  // Hangul Jamo Extended-A
        '\u{D7B0}'..='\u{D7FF}' |  // Hangul Jamo Extended-B
        '\u{3130}'..='\u{318F}'    // Hangul Compatibility Jamo
    )
}

/// Checks if a character is Japanese Hiragana
pub fn is_hiragana(ch: char) -> bool {
    matches!(ch, '\u{3040}'..='\u{309F}')
}

/// Checks if a character is Japanese Katakana
pub fn is_katakana(ch: char) -> bool {
    matches!(ch, '\u{30A0}'..='\u{30FF}' | '\u{31F0}'..='\u{31FF}')
}

/// Checks if a character is any East Asian script
/// (CJK ideographs, Hiragana, Katakana, or Hangul)
pub fn is_east_asian(ch: char) -> bool {
    is_cjk(ch) || is_hiragana(ch) || is_katakana(ch) || is_hangul(ch)
}
/// Normalizes text for tokenization.
///
/// Steps:
/// 1. Lowercase Latin characters
/// 2. Normalize punctuation
/// 3. Collapse whitespace
pub fn normalize(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut prev_whitespace = true; // Start true to trim leading space

    for ch in text.chars() {
        // Ellipsis expands to multiple chars, handle separately
        if ch == '\u{2026}' {
            result.push_str("...");
            prev_whitespace = false;
            continue;
        }

        let normalized = normalize_char(ch);

        // Collapse consecutive whitespace into single space
        if normalized.is_whitespace() {
            if !prev_whitespace {
                result.push(' ');
                prev_whitespace = true;
            }
        } else {
            result.push(normalized);
            prev_whitespace = false;
        }
    }

    // Trim trailing space
    if result.ends_with(' ') {
        result.pop();
    }

    result
}

/// Normalizes fancy quotes to ASCII quotes
fn normalize_quote(ch: char) -> Option<char> {
    match ch {
        // Single quotes: ' ' ‚ ‛
        '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => Some('\''),
        // Double quotes: " " „ ‟
        '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => Some('"'),
        _ => None,
    }
}

/// Normalizes dashes to ASCII hyphen
fn normalize_dash(ch: char) -> Option<char> {
    match ch {
        // Hyphen, non-breaking hyphen, figure dash, en dash, em dash, horizontal bar
        '\u{2010}'..='\u{2015}' => Some('-'),
        _ => None,
    }
}

/// Normalizes whitespace variants to ASCII space
fn normalize_whitespace_char(ch: char) -> Option<char> {
    match ch {
        '\u{00A0}' |                  // Non-breaking space
        '\u{2000}'..='\u{200A}' |     // Various typographic spaces
        '\u{202F}' |                  // Narrow no-break space
        '\u{205F}' |                  // Medium mathematical space
        '\u{3000}' => Some(' '),      // Ideographic space (全角空格)
        _ => None,
    }
}

/// Lowercases European accented letters (German, French, Polish)
fn lowercase_european(ch: char) -> Option<char> {
    match ch {
        // German
        'Ä' => Some('ä'),
        'Ö' => Some('ö'),
        'Ü' => Some('ü'),

        // French
        'À' => Some('à'),
        'Â' => Some('â'),
        'Æ' => Some('æ'),
        'Ç' => Some('ç'),
        'È' => Some('è'),
        'É' => Some('é'),
        'Ê' => Some('ê'),
        'Ë' => Some('ë'),
        'Î' => Some('î'),
        'Ï' => Some('ï'),
        'Ô' => Some('ô'),
        'Œ' => Some('œ'),
        'Ù' => Some('ù'),
        'Û' => Some('û'),
        'Ÿ' => Some('ÿ'),

        // Polish
        'Ą' => Some('ą'),
        'Ć' => Some('ć'),
        'Ę' => Some('ę'),
        'Ł' => Some('ł'),
        'Ń' => Some('ń'),
        'Ó' => Some('ó'),
        'Ś' => Some('ś'),
        'Ź' => Some('ź'),
        'Ż' => Some('ż'),

        _ => None,
    }
}

/// Normalizes a single character, returns the normalized character
#[inline]
fn normalize_char(ch: char) -> char {
    // ASCII uppercase -> lowercase
    if ch.is_ascii_uppercase() {
        return ch.to_ascii_lowercase();
    }

    // Try each normalization in order
    if let Some(c) = normalize_quote(ch) {
        return c;
    }
    if let Some(c) = normalize_dash(ch) {
        return c;
    }
    if let Some(c) = normalize_whitespace_char(ch) {
        return c;
    }
    if let Some(c) = lowercase_european(ch) {
        return c;
    }

    // No normalization needed
    ch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_lowercase() {
        assert_eq!(normalize("HELLO WORLD"), "hello world");
    }

    #[test]
    fn test_european_lowercase() {
        assert_eq!(normalize("ÄÖÜÉÇ"), "äöüéç");
        assert_eq!(normalize("ŁŃŚŹŻ"), "łńśźż");
    }

    #[test]
    fn test_quotes_normalized() {
        assert_eq!(normalize("'hello' \"world\""), "'hello' \"world\"");
    }

    #[test]
    fn test_dashes_normalized() {
        assert_eq!(normalize("a–b—c"), "a-b-c");
    }

    #[test]
    fn test_whitespace_collapsed() {
        assert_eq!(normalize("hello   world"), "hello world");
        assert_eq!(normalize("  hello  "), "hello");
        assert_eq!(normalize("a\u{3000}b"), "a b"); // Ideographic space
    }

    #[test]
    fn test_mixed_content() {
        assert_eq!(
            normalize("  HELLO   'World'   你好…  "),
            "hello 'world' 你好..."
        );
    }

    #[test]
    fn test_ellipsis_expanded() {
        assert_eq!(normalize("wait…what"), "wait...what");
        assert_eq!(normalize("…"), "...");
        assert_eq!(normalize("end…"), "end...");
        assert_eq!(normalize("…start"), "...start");
    }

    #[test]
    fn test_multiple_spaces_collapsed() {
        assert_eq!(normalize("hello   world"), "hello world");
        assert_eq!(normalize("a     b     c"), "a b c");
    }

    #[test]
    fn test_newlines_collapsed() {
        assert_eq!(normalize("hello\nworld"), "hello world");
        assert_eq!(normalize("hello\n\n\nworld"), "hello world");
    }

    #[test]
    fn test_tabs_collapsed() {
        assert_eq!(normalize("hello\tworld"), "hello world");
        assert_eq!(normalize("hello\t\t\tworld"), "hello world");
    }

    #[test]
    fn test_mixed_whitespace_collapsed() {
        assert_eq!(normalize("a \n\t b"), "a b");
    }

    #[test]
    fn test_leading_trailing_whitespace_trimmed() {
        assert_eq!(normalize("  hello  "), "hello");
        assert_eq!(normalize("\n\nhello\n\n"), "hello");
    }

    #[test]
    fn test_special_whitespace_normalized() {
        assert_eq!(normalize("a\u{00A0}b"), "a b"); // NBSP
        assert_eq!(normalize("a\u{3000}b"), "a b"); // Ideographic space
        assert_eq!(normalize("a\u{2003}b"), "a b"); // Em space
        assert_eq!(normalize("a\u{202F}b"), "a b"); // Narrow NBSP
    }

    #[test]
    fn test_normalize_char_ascii() {
        assert_eq!(normalize_char('A'), 'a');
        assert_eq!(normalize_char('Z'), 'z');
        assert_eq!(normalize_char('a'), 'a'); // Already lowercase
    }

    #[test]
    fn test_normalize_char_quotes() {
        assert_eq!(normalize_char('\u{2018}'), '\''); // '
        assert_eq!(normalize_char('\u{2019}'), '\''); // '
        assert_eq!(normalize_char('\u{201C}'), '"'); // "
        assert_eq!(normalize_char('\u{201D}'), '"'); // "
    }

    #[test]
    fn test_normalize_char_dashes() {
        assert_eq!(normalize_char('\u{2013}'), '-'); // en dash
        assert_eq!(normalize_char('\u{2014}'), '-'); // em dash
    }

    #[test]
    fn test_normalize_char_european() {
        assert_eq!(normalize_char('É'), 'é');
        assert_eq!(normalize_char('Ü'), 'ü');
        assert_eq!(normalize_char('Ł'), 'ł');
        assert_eq!(normalize_char('é'), 'é'); // Already lowercase
    }

    #[test]
    fn test_complex_mixed_content() {
        assert_eq!(
            normalize("  HELLO   'World'   你好…  "),
            "hello 'world' 你好..."
        );
    }

    #[test]
    fn test_realistic_sentence() {
        assert_eq!(
            normalize("The café's \"special\" costs €10–€15."),
            "the café's \"special\" costs €10-€15."
        );
    }

    #[test]
    fn test_wikipedia_like_content() {
        assert_eq!(
            normalize("Albert Einstein (1879–1955) was a German-born physicist…"),
            "albert einstein (1879-1955) was a german-born physicist..."
        );
    }

    #[test]
    fn test_multilingual_content() {
        assert_eq!(normalize("Ü北京 — PARIS — 東京"), "ü北京 - paris - 東京");
    }

    #[test]
    fn test_is_cjk() {
        // Common CJK characters
        assert!(is_cjk('中'));
        assert!(is_cjk('国'));
        assert!(is_cjk('字'));

        // CJK Extension B (rare characters)
        assert!(is_cjk('\u{20000}'));

        // Non-CJK
        assert!(!is_cjk('a'));
        assert!(!is_cjk('あ')); // Hiragana
        assert!(!is_cjk('ア')); // Katakana
        assert!(!is_cjk('한')); // Hangul
    }

    #[test]
    fn test_is_hangul() {
        assert!(is_hangul('한'));
        assert!(is_hangul('글'));
        assert!(is_hangul('\u{AC00}')); // First Hangul syllable

        assert!(!is_hangul('中'));
        assert!(!is_hangul('a'));
    }

    #[test]
    fn test_is_hiragana() {
        assert!(is_hiragana('あ'));
        assert!(is_hiragana('ひ'));
        assert!(is_hiragana('ん'));

        assert!(!is_hiragana('ア')); // Katakana
        assert!(!is_hiragana('中'));
    }

    #[test]
    fn test_is_katakana() {
        assert!(is_katakana('ア'));
        assert!(is_katakana('カ'));
        assert!(is_katakana('ン'));

        assert!(!is_katakana('あ')); // Hiragana
        assert!(!is_katakana('中'));
    }

    #[test]
    fn test_is_east_asian() {
        assert!(is_east_asian('中')); // CJK
        assert!(is_east_asian('あ')); // Hiragana
        assert!(is_east_asian('ア')); // Katakana
        assert!(is_east_asian('한')); // Hangul

        assert!(!is_east_asian('a'));
        assert!(!is_east_asian('é'));
    }

    #[test]
    fn test_cjk_preserved() {
        assert_eq!(normalize("你好世界"), "你好世界");
        assert_eq!(normalize("中文测试"), "中文测试");
    }

    #[test]
    fn test_japanese_preserved() {
        assert_eq!(normalize("ひらがな"), "ひらがな");
        assert_eq!(normalize("カタカナ"), "カタカナ");
        assert_eq!(normalize("漢字とひらがな"), "漢字とひらがな");
    }

    #[test]
    fn test_korean_preserved() {
        assert_eq!(normalize("한글"), "한글");
        assert_eq!(normalize("안녕하세요"), "안녕하세요");
    }

    #[test]
    fn test_mixed_scripts() {
        assert_eq!(normalize("Hello 世界"), "hello 世界");
        assert_eq!(
            normalize("Bonjour 你好 こんにちは"),
            "bonjour 你好 こんにちは"
        );
        assert_eq!(
            normalize("MIXED中文English日本語"),
            "mixed中文english日本語"
        );
    }
}
