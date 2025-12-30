//! Text normalization
//!
//! - Unicode normalization (NFC)
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
