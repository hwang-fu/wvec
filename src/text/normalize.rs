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
        // Handle ellipsis separately (expands to multiple chars)
        if ch == '\u{2026}' {
            result.push_str("...");
            prev_whitespace = false;
            continue;
        }

        let normalized = match ch {
            // Lowercase ASCII
            'A'..='Z' => ch.to_ascii_lowercase(),

            // Normalize quotes
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => '\'',
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',

            // Normalize whitespace variants
            '\u{00A0}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => ' ',

            // Lowercase German umlauts
            'Ä' => 'ä',
            'Ö' => 'ö',
            'Ü' => 'ü',

            // French accented letters
            'À' => 'à',
            'Â' => 'â',
            'Æ' => 'æ',
            'Ç' => 'ç',
            'È' => 'è',
            'É' => 'é',
            'Ê' => 'ê',
            'Ë' => 'ë',
            'Î' => 'î',
            'Ï' => 'ï',
            'Ô' => 'ô',
            'Œ' => 'œ',
            'Ù' => 'ù',
            'Û' => 'û',
            'Ÿ' => 'ÿ',

            // Polish special letters
            'Ą' => 'ą',
            'Ć' => 'ć',
            'Ę' => 'ę',
            'Ł' => 'ł',
            'Ń' => 'ń',
            'Ó' => 'ó',
            'Ś' => 'ś',
            'Ź' => 'ź',
            'Ż' => 'ż',

            // Everything else as-is
            _ => ch,
        };

        // Collapse whitespace
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
