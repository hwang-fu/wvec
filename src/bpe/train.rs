//! BPE Training Algorithm
//!
//! Learns vocabulary from corpus by iteratively merging frequent pairs.

use std::collections::HashMap;

use crate::bpe::types::BpeTokenId;

/// Counts adjacent token pairs across all words.
fn count_pairs(
    word_tokens: &[Vec<BpeTokenId>],
    word_counts: &[u32],
) -> HashMap<(BpeTokenId, BpeTokenId), u64> {
    let mut counts: HashMap<(BpeTokenId, BpeTokenId), u64> = HashMap::new();

    for (tokens, &word_count) in word_tokens.iter().zip(word_counts.iter()) {
        for window in tokens.windows(2) {
            let pair = (window[0], window[1]);
            *counts.entry(pair).or_insert(0) += word_count as u64;
        }
    }

    counts
}
