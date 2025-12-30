//! BPE Encoding/Decoding
//!
//! Converts text to token IDs and back.

use crate::bpe::{
    types::{BpeTokenId, UNK_ID, UNK_TOKEN},
    vocab::Vocabulary,
};

/// Encodes a pre-token into a sequence of BPE token IDs.
///
/// Applies learned merge rules in priority order.
pub fn encode(vocab: &Vocabulary, pretoken: &str) -> Vec<BpeTokenId> {
    // TODO: Implement
    // 1. Convert pretoken to character-level token IDs
    // 2. Iteratively apply merge rules from vocab.pairs()
    // 3. Return final sequence
    panic!("TODO: encode")
}

/// Decodes a sequence of token IDs back to a string.
pub fn decode(vocab: &Vocabulary, ids: &[BpeTokenId]) -> String {
    let mut result = String::new();

    for &id in ids.iter() {
        match vocab.get_token(id) {
            Some(token) => result.push_str(token),
            None => result.push_str(UNK_TOKEN),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_basic() {
        // TODO: Add tests
    }

    #[test]
    fn test_encode_basic() {
        // TODO: Add tests
    }
}
