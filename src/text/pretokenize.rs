//! Pre-tokenization: splitting text into initial tokens
//!
//! Language-aware splitting:
//! - English/German: whitespace + punctuation boundaries
//! - Chinese: character-level (each character is a token)

use crate::text::normalize::is_cjk;

/// A pre-token with its text content
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreToken {
    pub text: String,
}

/// Pre-tokenizes text based on language characteristics.
///
/// - Latin text: split on whitespace and punctuation
/// - CJK text: each character becomes a separate token
/// - Mixed text: handled appropriately
pub fn pretokenize(text: &str) -> Vec<PreToken> {
    let estimated_tokens = text.len() / 4 + 1;
    let mut tokens = Vec::with_capacity(estimated_tokens);
    let mut current = String::new();

    for ch in text.chars() {
        let ch_is_cjk = is_cjk(ch);

        // CJK characters: each is its own token
        if ch_is_cjk {
            // Flush any accumulated Latin text
            flush_token(&mut tokens, &mut current);

            // Add CJK char as its own token
            tokens.push(PreToken {
                text: ch.to_string(),
            });
            continue;
        }

        // Whitespace: flush current token
        if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(PreToken {
                    text: std::mem::take(&mut current),
                });
            }
            continue;
        }

        // Punctuation: separate token (unless it's an apostrophe in a word)
        if ch.is_ascii_punctuation() && ch != '\'' {
            if !current.is_empty() {
                tokens.push(PreToken {
                    text: std::mem::take(&mut current),
                });
            }
            tokens.push(PreToken {
                text: ch.to_string(),
            });
            continue;
        }

        // Regular character: accumulate
        current.push(ch);
    }

    // Flush remaining
    if !current.is_empty() {
        tokens.push(PreToken { text: current });
    }

    tokens
}

#[inline]
fn flush_token(tokens: &mut Vec<PreToken>, current: &mut String) {
    if !current.is_empty() {
        tokens.push(PreToken {
            text: std::mem::take(current),
        });
    }
}
