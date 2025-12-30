//! BPE Encoding
//!
//! Converts text to token IDs.

use crate::bpe::{types::BpeTokenId, vocab::Vocabulary};

/// Encodes a pre-token into a sequence of BPE token IDs.
///
/// Applies learned merge rules in priority order.
///
/// # Algorithm
///
/// 1. Convert each character to its token ID (unknown chars → UNK_ID)
/// 2. Apply merge rules in priority order (most frequent merges first)
///
/// # Example
///
/// ```text
/// vocab: {"h": 4, "i": 5, "hi": 6}
/// merge rules: [(4, 5) -> 6]
///
/// encode(vocab, "hi"):
///   Step 1: "hi" -> chars ['h', 'i'] -> IDs [4, 5]
///   Step 2: apply merge (4,5)->6 -> [6]
///   Result: [6]
///
/// encode(vocab, "hih"):
///   Step 1: "hih" -> ['h', 'i', 'h'] -> [4, 5, 4]
///   Step 2: apply merge (4,5)->6 -> [6, 4]
///   Result: [6, 4]
/// ```
pub fn encode(vocab: &Vocabulary, pretoken: &str) -> Vec<BpeTokenId> {
    if pretoken.is_empty() {
        return Vec::new();
    }

    let mut ids: Vec<BpeTokenId> = pretoken
        .chars()
        .map(|ch| vocab.get_id(&ch.to_string()))
        .collect();

    for pair in vocab.pairs() {
        apply_merge(&mut ids, pair.left, pair.right, pair.id);
    }

    ids
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

    use crate::bpe::{decode::decode, types::UNK_ID};

    #[test]
    fn test_encode_basic() {
        use crate::bpe::train::train;

        // Train a small vocab
        let pretokens = ["ab", "ab", "ab"];
        let vocab = train(pretokens.into_iter(), 10);

        // Encode should produce valid IDs
        let ids = encode(&vocab, "ab");
        assert!(!ids.is_empty());

        // Round-trip: decode should give back "ab"
        let decoded = decode(&vocab, &ids);
        assert_eq!(decoded, "ab");
    }

    #[test]
    fn test_encode_empty() {
        let vocab = Vocabulary::new();
        let ids = encode(&vocab, "");
        assert!(ids.is_empty());
    }

    #[test]
    fn test_encode_unknown_char() {
        let vocab = Vocabulary::new(); // Only has special tokens
        let ids = encode(&vocab, "x");
        assert_eq!(ids, vec![UNK_ID]);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        use crate::bpe::train::train;

        let pretokens = ["hello", "hello", "world"];
        let vocab = train(pretokens.into_iter(), 20);

        let ids = encode(&vocab, "hello");
        let decoded = decode(&vocab, &ids);
        assert_eq!(decoded, "hello");
    }

    #[test]
    fn test_encode_applies_merges() {
        use crate::bpe::train::train;

        // "aa" repeated -> should learn to merge 'a'+'a' -> "aa"
        let pretokens = ["aa", "aa", "aa", "aa"];
        let vocab = train(pretokens.into_iter(), 10);

        let ids = encode(&vocab, "aa");
        // After merge, "aa" should be 1 token (not 2)
        assert_eq!(ids.len(), 1);
    }
}
