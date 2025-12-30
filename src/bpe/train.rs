//! BPE Training Algorithm
//!
//! Learns vocabulary from corpus by iteratively merging frequent pairs.

use std::collections::HashMap;

use crate::{
    bpe::{
        types::BpeTokenId,
        vocab::{self, Vocabulary},
    },
    text::pretokenize,
};

/// Trains a BPE vocabulary from pre-tokenized text.
pub fn train<'a, I>(pretokens: I, vocab_size: usize) -> Vocabulary
where
    I: Iterator<Item = &'a str>,
{
    let mut vocab = Vocabulary::new();

    let mut sequences: Vec<Vec<BpeTokenId>> = Vec::new();
    let mut freqs: Vec<u32> = Vec::new();

    let mut seq_freq: HashMap<String, u32> = HashMap::new();
    for pretoken in pretokens {
        if !pretoken.is_empty() {
            *seq_freq.entry(pretoken.to_string()).or_insert(0) += 1;
        }
    }

    for (seq, freq) in seq_freq {
        let mut tokens = Vec::new();
        for ch in seq.chars() {
            let id = vocab.add_token(ch.to_string());
            tokens.push(id);
        }
        if !tokens.is_empty() {
            sequences.push(tokens);
            freqs.push(freq);
        }
    }

    while vocab.len() < vocab_size {
        let pair_counts = count_pairs(&sequences, &freqs);
        if pair_counts.is_empty() {
            break;
        }

        let ((left, right), _) = pair_counts.iter().max_by_key(|(_, count)| *count).unwrap();
        let left = *left;
        let right = *right;

        let left_token = vocab.get_token(left).unwrap();
        let right_token = vocab.get_token(right).unwrap();
        let pair = format!("{}{}", left_token, right_token);
        let pair_id = vocab.add_token(pair);

        vocab.add_pair(left, right, pair_id);

        apply_merge(&mut sequences, left, right, pair_id);
    }

    vocab
}

/// Counts adjacent token pair frequencies across all sequences.
/// Returns a map from token pairs to their total frequency across the corpus.
/// # Example
///
/// ```text
/// Corpus: "hello hello world"
///
/// After tokenization:
///   sequences:   [[7,4,11,11,14], [22,14,17,11,3]]   ("hello", "world")
///   frequencies: [2, 1]                              (hello x 2, world x 1)
///
/// Pair counting for "hello" (freq=2):
///   (7,4)    +2
///   (4,11)   +2
///   (11,11)  +2
///   (11,14)  +2
///
/// Pair counting for "world" (freq=1):
///   (22,14)  +1
///   (14,17)  +1
///   (17,11)  +1
///   (11,3)   +1
///
/// Result: {(7,4): 2, (4,11): 2, (11,11): 2, (11,14): 2,
///          (22,14): 1, (14,17): 1, (17,11): 1, (11,3): 1}
/// ```
fn count_pairs(
    sequences: &[Vec<BpeTokenId>],
    freqs: &[u32],
) -> HashMap<(BpeTokenId, BpeTokenId), u64> {
    let mut counts: HashMap<(BpeTokenId, BpeTokenId), u64> = HashMap::new();

    for (seq, freq) in sequences.iter().zip(freqs.iter()) {
        for window in seq.windows(2) {
            let pair = (window[0], window[1]);
            *counts.entry(pair).or_insert(0) += (*freq) as u64;
        }
    }

    counts
}

/// Applies a merge rule to all sequences, combining adjacent token pairs.
///
/// When BPE finds the most frequent pair, this function replaces all
/// occurrences of that pair with a single merged token. This is done
/// in-place to avoid allocation.
///
/// # Example
///
/// ```text
/// Before:
/// - sequences = [[7, 4, 11, 11, 14], [7, 4, 11, 15]]
/// - left=11
/// - right=11
/// - merged_id=256
///
/// Processing [7, 4, 11, 11, 14]:
///   i=0: (7,4)   ≠ (11,11) → i++
///   i=1: (4,11)  ≠ (11,11) → i++
///   i=2: (11,11) = (11,11) → replace with 256, remove next
///   Result: [7, 4, 256, 14]
///
/// Processing [7, 4, 11, 15]:
///   No (11,11) pair found
///   Result: [7, 4, 11, 15] (unchanged)
///
/// After:
/// - sequences = [[7, 4, 256, 14], [7, 4, 11, 15]]
/// ```
fn apply_merge(
    sequences: &mut [Vec<BpeTokenId>],
    left: BpeTokenId,
    right: BpeTokenId,
    merged_id: BpeTokenId,
) {
    for seq in sequences.iter_mut() {
        let mut i = 0;
        while i + 1 < seq.len() {
            if seq[i] == left && seq[i + 1] == right {
                seq[i] = merged_id;
                seq.remove(i + 1);
                // Do NOT increment i: check if new token forms another pair
            } else {
                i += 1;
            }
        }
    }
}
