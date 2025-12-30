//! BPE Decoding
//!
//! Converts token IDs to text.

use crate::bpe::{
    types::{BpeTokenId, UNK_TOKEN},
    vocab::Vocabulary,
};

/// Decodes a sequence of token IDs back to a string.
///
/// Unknown IDs are replaced with "[UNK]".
///
/// # Example
///
/// ```text
/// vocab: {4: "h", 5: "i", 6: "hi"}
///
/// decode(vocab, [4, 5])  -> "hi"
/// decode(vocab, [6])     -> "hi"
/// decode(vocab, [4, 999]) -> "h[UNK]"  (999 not in vocab)
/// decode(vocab, [])      -> ""
/// ```
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
        let mut vocab = Vocabulary::new();
        let h = vocab.add_token("h".to_string());
        let i = vocab.add_token("i".to_string());

        let result = decode(&vocab, &[h, i]);
        assert_eq!(result, "hi");
    }

    #[test]
    fn test_decode_empty() {
        let vocab = Vocabulary::new();
        let result = decode(&vocab, &[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_decode_unknown_id() {
        let vocab = Vocabulary::new();
        let result = decode(&vocab, &[999]); // ID doesn't exist
        assert_eq!(result, "[UNK]");
    }

    #[test]
    fn test_decode_merged_token() {
        let mut vocab = Vocabulary::new();
        let hello = vocab.add_token("hello".to_string());

        let result = decode(&vocab, &[hello]);
        assert_eq!(result, "hello");
    }
}
