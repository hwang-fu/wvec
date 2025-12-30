//! Pre-tokenization: splitting text into initial tokens
//!
//! Language-aware splitting:
//! - English/German: whitespace + punctuation boundaries
//! - Chinese: character-level (each character is a token)

/// A pre-token with its text content
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreToken {
    pub text: String,
}
