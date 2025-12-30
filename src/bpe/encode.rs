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
    if pretoken.is_empty() {
        return Vec::new();
    }

    // Step 1: Convert to character-level token IDs
    let mut ids: Vec<BpeTokenId> = pretoken
        .chars()
        .map(|ch| vocab.get_id(&ch.to_string()))
        .collect();

    // Step 2: Apply merge rules in priority order
    for pair in vocab.pairs() {
        apply_merge(&mut ids, pair.left, pair.right, pair.id);
    }

    ids
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

/// Applies a single merge rule to a token sequence.
///
/// Replaces all adjacent (left, right) pairs with merged_id.
fn apply_merge(
    ids: &mut Vec<BpeTokenId>,
    left: BpeTokenId,
    right: BpeTokenId,
    merged_id: BpeTokenId,
) {
    let mut i = 0;
    while i + 1 < ids.len() {
        if ids[i] == left && ids[i + 1] == right {
            ids[i] = merged_id;
            ids.remove(i + 1);
            // Don't increment: merged token might form new pair
        } else {
            i += 1;
        }
    }
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

    #[test]
    fn test_encode_basic() {
        // TODO: Add tests
    }
}
