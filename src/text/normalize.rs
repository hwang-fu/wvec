//! Text normalization
//!
//! - Unicode normalization (NFC)
//! - Case folding (lowercase for Latin scripts)
//! - Punctuation normalization
//! - Whitespace normalization

/// Checks if a character is CJK (Chinese/Japanese/Korean)
pub fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' |     // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |     // CJK Unified Ideographs Extension A
        '\u{20000}'..='\u{2A6DF}' |   // CJK Unified Ideographs Extension B
        '\u{2A700}'..='\u{2B73F}' |   // CJK Unified Ideographs Extension C
        '\u{2B740}'..='\u{2B81F}' |   // CJK Unified Ideographs Extension D
        '\u{F900}'..='\u{FAFF}' |     // CJK Compatibility Ideographs
        '\u{2F800}'..='\u{2FA1F}'     // CJK Compatibility Ideographs Supplement
    )
}
