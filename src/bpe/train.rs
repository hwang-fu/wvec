//! BPE Training Algorithm
//!
//! Learns vocabulary from corpus by iteratively merging frequent pairs.

use std::collections::HashMap;

use crate::bpe::{types::BpeTokenId, vocab::Vocabulary};

/// Trains a BPE vocabulary from pre-tokenized text.
///
/// # Arguments
///
/// * `pretokens`  - Iterator over pre-tokenized words (e.g., from whitespace splitting)
/// * `target_max_vocab_size` - Target vocabulary size (including special tokens)
///
/// # Returns
///
/// A trained `Vocabulary` with token mappings and merge rules.
///
/// # Example
///
/// ```text
/// Input:
/// - pretokens = ["hello", "hello", "world"]
/// - target_max_vocab_size = 10
///
/// Step 1: Count pretoken frequencies
///   pretoken_freqs = {"hello": 2, "world": 1}
///
/// Step 2: Initialize character-level tokens
///   vocab:        {"h":4, "e":5, "l":6, "o":7, "w":8, "r":9, "d":10}
///   sequences:    [[4,5,6,6,7], [8,7,9,6,10]]     ("hello",     "world")
///   freqs:        [2, 1]                          ("hello" x 2, "world" x 1)
///
/// Step 3: Iterative merging
///   Iteration 1:
///     pair_counts:        {(6,6): 2, (4,5): 2, (5,6): 2, ...}
///     most_frequent:      (6,6) -> merge into "ll" (id=11)
///     sequences becomes:  [[4,5,11,7], [8,7,9,6,10]]
///     vocab:              {"h":4, "e":5, "l":6, "o":7, "w":8, "r":9, "d":10, "ll":11}
///
///   Iteration 2:
///     most_frequent:      (5,11) -> merge into "ell" (id=12)
///     sequences becomes:  [[4,12,7], [8,7,9,6,10]]
///
///   ... continues until vocab.len() >= target_max_vocab_size
/// ```
pub fn train<'a, I>(pretokens: I, target_max_vocab_size: usize) -> Vocabulary
where
    I: Iterator<Item = &'a str>,
{
    let mut vocab = Vocabulary::new();

    // Step 1: Count frequency of each unique pretoken
    let pretoken_freqs = count_pretoken_freqs(pretokens);

    // Step 2: Initialize character-level tokenization
    let (mut sequences, freqs) = init_char_sequences(&pretoken_freqs, &mut vocab);

    // Step 3: Iteratively merge most frequent pairs
    while vocab.len() < target_max_vocab_size {
        let pair_counts = count_pair_freqs(&sequences, &freqs);
        if pair_counts.is_empty() {
            break; // No more pairs to merge
        }

        let (left, right) = find_most_frequent_pair(&pair_counts);
        let merged_id = merge_tokens(&mut vocab, left, right);
        apply_merge(&mut sequences, left, right, merged_id);
    }

    vocab
}

/// Counts frequency of each unique pretoken in the corpus.
///
/// # Example
///
/// ```text
/// Input:  ["hello", "world", "hello", "hello"]
/// Output: {"hello": 3, "world": 1}
/// ```
fn count_pretoken_freqs<'a, I>(pretokens: I) -> HashMap<String, u32>
where
    I: Iterator<Item = &'a str>,
{
    let mut freqs: HashMap<String, u32> = HashMap::new();

    for pretoken in pretokens {
        if !pretoken.is_empty() {
            *freqs.entry(pretoken.to_string()).or_insert(0) += 1;
        }
    }

    freqs
}

/// Initializes character-level token sequences from pretokens.
///
/// Each character becomes a separate token ID. Populates the vocabulary
/// with all unique characters encountered.
///
/// # Example
///
/// ```text
/// Input:
/// - pretoken_freqs = {"hi": 2, "ho": 1}
///
/// Output:
///   vocab gets: {h: 4, i: 5, o: 6}  (0-3 reserved for special tokens)
///   sequences = [[4, 5], [4, 6]]    (hi, ho)
///   freqs = [2, 1]
/// ```
fn init_char_sequences(
    pretoken_freqs: &HashMap<String, u32>,
    vocab: &mut Vocabulary,
) -> (Vec<Vec<BpeTokenId>>, Vec<u32>) {
    let mut sequences = Vec::with_capacity(pretoken_freqs.len());
    let mut freqs = Vec::with_capacity(pretoken_freqs.len());

    for (pretoken, &freq) in pretoken_freqs {
        let token_ids: Vec<BpeTokenId> = pretoken
            .chars()
            .map(|ch| vocab.add_token(ch.to_string()))
            .collect();

        if !token_ids.is_empty() {
            sequences.push(token_ids);
            freqs.push(freq);
        }
    }

    (sequences, freqs)
}

/// Counts adjacent token pair frequencies across all sequences.
///
/// # Example
///
/// ```text
/// Input:
/// - sequences = [[7,4,11,11,14], [22,14,17,11,3]]  (hello, world)
/// - freqs     = [2, 1]                             (hello x 2, world x 1)
///
/// Counting:
///   "hello" (freq=2): (7,4)+2, (4,11)+2, (11,11)+2, (11,14)+2
///   "world" (freq=1): (22,14)+1, (14,17)+1, (17,11)+1, (11,3)+1
///
/// Output:
///   { (7,  4) : 2, (4,  11): 2, (11, 11): 2, (11, 14): 2,
///     (22, 14): 1, (14, 17): 1, (17, 11): 1, (11, 3): 1 }
/// ```
fn count_pair_freqs(
    sequences: &[Vec<BpeTokenId>],
    freqs: &[u32],
) -> HashMap<(BpeTokenId, BpeTokenId), u64> {
    let mut pair_counts: HashMap<(BpeTokenId, BpeTokenId), u64> = HashMap::new();

    for (seq, &freq) in sequences.iter().zip(freqs.iter()) {
        for window in seq.windows(2) {
            let pair = (window[0], window[1]);
            *pair_counts.entry(pair).or_insert(0) += freq as u64;
        }
    }

    pair_counts
}

/// Finds the most frequent token pair.
/// Panics if `pair_counts` is empty.
fn find_most_frequent_pair(
    pair_counts: &HashMap<(BpeTokenId, BpeTokenId), u64>,
) -> (BpeTokenId, BpeTokenId) {
    pair_counts
        .iter()
        .max_by_key(|((_l, _r), count)| *count)
        .map(|(&pair, _count)| pair)
        .expect("pair_counts should not be empty")
}

/// Creates a merged token and records the merge rule.
///
/// # Example
///
/// ```text
/// vocab contains: {4: "h", 5: "e"}
/// left=4, right=5
///
/// Result:
///   New token "he" added with id=11
///   Merge rule (4, 5) -> 11 recorded
///   Returns: 11
/// ```
fn merge_tokens(vocab: &mut Vocabulary, left: BpeTokenId, right: BpeTokenId) -> BpeTokenId {
    let left_str = vocab.get_token(left).unwrap();
    let right_str = vocab.get_token(right).unwrap();
    let merged_str = format!("{}{}", left_str, right_str);

    let merged_id = vocab.add_token(merged_str);
    vocab.add_pair(left, right, merged_id);

    merged_id
}

/// Applies a merge rule to all sequences in-place.
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
///   i=0: (7,4)   != (11,11) -> i++
///   i=1: (4,11)  != (11,11) -> i++
///   i=2: (11,11) == (11,11) -> replace with 256, remove next
///   Result: [7, 4, 256, 14]
///
/// Processing [7, 4, 11, 15]: No (11,11) pair found, unchanged
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
    for seq in sequences {
        let mut i = 0;
        while i + 1 < seq.len() {
            if seq[i] == left && seq[i + 1] == right {
                seq[i] = merged_id;
                seq.remove(i + 1);
                // Don't increment i: new token might form another pair
            } else {
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_pretoken_freqs_basic() {
        let pretokens = ["hello", "world", "hello", "hello"];
        let freqs = count_pretoken_freqs(pretokens.into_iter());

        assert_eq!(freqs.get("hello"), Some(&3));
        assert_eq!(freqs.get("world"), Some(&1));
        assert_eq!(freqs.len(), 2);
    }

    #[test]
    fn test_count_pretoken_freqs_empty() {
        let pretokens: [&str; 0] = [];
        let freqs = count_pretoken_freqs(pretokens.into_iter());

        assert!(freqs.is_empty());
    }

    #[test]
    fn test_count_pretoken_freqs_skips_empty_strings() {
        let pretokens = ["hello", "", "world", ""];
        let freqs = count_pretoken_freqs(pretokens.into_iter());

        assert_eq!(freqs.len(), 2);
        assert!(!freqs.contains_key(""));
    }

    #[test]
    fn test_count_pretoken_freqs_unicode() {
        let pretokens = ["你好", "世界", "你好"];
        let freqs = count_pretoken_freqs(pretokens.into_iter());

        assert_eq!(freqs.get("你好"), Some(&2));
        assert_eq!(freqs.get("世界"), Some(&1));
    }

    #[test]
    fn test_init_char_sequences_basic() {
        let mut pretoken_freqs = HashMap::new();
        pretoken_freqs.insert("ab".to_string(), 2);

        let mut vocab = Vocabulary::new();
        let (sequences, freqs) = init_char_sequences(&pretoken_freqs, &mut vocab);

        assert_eq!(sequences.len(), 1);
        assert_eq!(freqs.len(), 1);
        assert_eq!(freqs[0], 2);
        assert_eq!(sequences[0].len(), 2); // 'a' and 'b'
    }

    #[test]
    fn test_init_char_sequences_multiple_words() {
        let mut pretoken_freqs = HashMap::new();
        pretoken_freqs.insert("hi".to_string(), 3);
        pretoken_freqs.insert("ho".to_string(), 1);

        let mut vocab = Vocabulary::new();
        let (sequences, freqs) = init_char_sequences(&pretoken_freqs, &mut vocab);

        assert_eq!(sequences.len(), 2);
        assert_eq!(freqs.len(), 2);
    }

    #[test]
    fn test_init_char_sequences_shared_chars() {
        let mut pretoken_freqs = HashMap::new();
        pretoken_freqs.insert("ab".to_string(), 1);
        pretoken_freqs.insert("ba".to_string(), 1);

        let mut vocab = Vocabulary::new();
        let (sequences, _freqs) = init_char_sequences(&pretoken_freqs, &mut vocab);

        // 'a' and 'b' should be added only once each
        // Special tokens (0-3) + 'a' + 'b' = 6 or just check unique chars
        let all_ids: std::collections::HashSet<_> = sequences.iter().flatten().collect();
        assert_eq!(all_ids.len(), 2); // Only 2 unique token IDs (a, b)
    }

    #[test]
    fn test_count_pair_freqs_basic() {
        let sequences = vec![vec![1, 2, 3]];
        let freqs = vec![1];

        let pair_counts = count_pair_freqs(&sequences, &freqs);

        assert_eq!(pair_counts.get(&(1, 2)), Some(&1));
        assert_eq!(pair_counts.get(&(2, 3)), Some(&1));
        assert_eq!(pair_counts.len(), 2);
    }

    #[test]
    fn test_count_pair_freqs_with_frequency() {
        let sequences = vec![vec![1, 2, 3]];
        let freqs = vec![5]; // Word appears 5 times

        let pair_counts = count_pair_freqs(&sequences, &freqs);

        assert_eq!(pair_counts.get(&(1, 2)), Some(&5));
        assert_eq!(pair_counts.get(&(2, 3)), Some(&5));
    }

    #[test]
    fn test_count_pair_freqs_multiple_sequences() {
        let sequences = vec![
            vec![1, 2], // "ab"
            vec![1, 2], // "ab" again
        ];
        let freqs = vec![2, 3]; // First word x2, second word x3

        let pair_counts = count_pair_freqs(&sequences, &freqs);

        assert_eq!(pair_counts.get(&(1, 2)), Some(&5)); // 2 + 3 = 5
    }

    #[test]
    fn test_count_pair_freqs_repeated_pair() {
        let sequences = vec![vec![1, 1, 1]]; // "aaa"
        let freqs = vec![1];

        let pair_counts = count_pair_freqs(&sequences, &freqs);

        assert_eq!(pair_counts.get(&(1, 1)), Some(&2)); // Two (1,1) pairs
    }

    #[test]
    fn test_count_pair_freqs_single_token() {
        let sequences = vec![vec![1]]; // Single character word
        let freqs = vec![1];

        let pair_counts = count_pair_freqs(&sequences, &freqs);

        assert!(pair_counts.is_empty()); // No pairs possible
    }

    #[test]
    fn test_count_pair_freqs_empty() {
        let sequences: Vec<Vec<BpeTokenId>> = vec![];
        let freqs: Vec<u32> = vec![];

        let pair_counts = count_pair_freqs(&sequences, &freqs);

        assert!(pair_counts.is_empty());
    }

    #[test]
    fn test_find_most_frequent_pair_basic() {
        let mut pair_counts = HashMap::new();
        pair_counts.insert((1, 2), 10);
        pair_counts.insert((3, 4), 5);
        pair_counts.insert((5, 6), 20);

        let (left, right) = find_most_frequent_pair(&pair_counts);

        assert_eq!((left, right), (5, 6));
    }

    #[test]
    fn test_find_most_frequent_pair_single() {
        let mut pair_counts = HashMap::new();
        pair_counts.insert((1, 2), 100);

        let (left, right) = find_most_frequent_pair(&pair_counts);

        assert_eq!((left, right), (1, 2));
    }

    #[test]
    #[should_panic(expected = "pair_counts should not be empty")]
    fn test_find_most_frequent_pair_empty_panics() {
        let pair_counts: HashMap<(BpeTokenId, BpeTokenId), u64> = HashMap::new();
        find_most_frequent_pair(&pair_counts);
    }

    #[test]
    fn test_apply_merge_basic() {
        let mut sequences = vec![vec![1, 2, 3]];
        apply_merge(&mut sequences, 1, 2, 99);

        assert_eq!(sequences[0], vec![99, 3]);
    }

    #[test]
    fn test_apply_merge_multiple_occurrences() {
        let mut sequences = vec![vec![1, 2, 1, 2]];
        apply_merge(&mut sequences, 1, 2, 99);

        assert_eq!(sequences[0], vec![99, 99]);
    }

    #[test]
    fn test_apply_merge_no_match() {
        let mut sequences = vec![vec![1, 2, 3]];
        apply_merge(&mut sequences, 5, 6, 99);

        assert_eq!(sequences[0], vec![1, 2, 3]); // Unchanged
    }

    #[test]
    fn test_apply_merge_adjacent_pairs() {
        // "aaa" = [1, 1, 1], merge (1, 1) -> 99
        let mut sequences = vec![vec![1, 1, 1]];
        apply_merge(&mut sequences, 1, 1, 99);

        // First merge: [99, 1], then check again: no more (1,1)
        assert_eq!(sequences[0], vec![99, 1]);
    }

    #[test]
    fn test_apply_merge_multiple_sequences() {
        let mut sequences = vec![vec![1, 2, 3], vec![1, 2, 4], vec![5, 6, 7]];
        apply_merge(&mut sequences, 1, 2, 99);

        assert_eq!(sequences[0], vec![99, 3]);
        assert_eq!(sequences[1], vec![99, 4]);
        assert_eq!(sequences[2], vec![5, 6, 7]); // Unchanged
    }

    #[test]
    fn test_train_basic() {
        let pretokens = ["ab", "ab", "ab"];
        let vocab = train(pretokens.into_iter(), 10);

        // Should have at least: special tokens + 'a' + 'b' + "ab"
        assert!(vocab.len() >= 7); // 4 special + a + b + ab
    }

    #[test]
    fn test_train_respects_vocab_size() {
        // "aabb" has only 2 unique chars: a, b
        // Initial vocab: 4 special + 2 chars = 6
        // Target: 8, so can do at most 2 merges
        let pretokens = ["aabb", "aabb"];
        let vocab = train(pretokens.into_iter(), 8);
        assert!(vocab.len() <= 8);
    }

    #[test]
    fn test_train_empty_input() {
        let pretokens: [&str; 0] = [];
        let vocab = train(pretokens.into_iter(), 100);

        // Should have only special tokens
        assert_eq!(vocab.len(), 4);
    }

    #[test]
    fn test_train_single_char_words() {
        let pretokens = ["a", "b", "c"];
        let vocab = train(pretokens.into_iter(), 10);

        // 4 special + 3 chars = 7, no merges possible
        assert_eq!(vocab.len(), 7);
    }

    #[test]
    fn test_train_merges_frequent_pairs() {
        // "aab" x3 and "aac" x1
        // (a,a) appears in both: 3 + 1 = 4
        // (a,b) appears only in "aab": 3
        // (a,c) appears only in "aac": 1
        // So (a,a) should be merged first
        let pretokens = ["aab", "aab", "aab", "aac"];
        let vocab = train(pretokens.into_iter(), 20);

        let has_aa =
            (0..vocab.len() as u32).any(|id| vocab.get_token(id).is_some_and(|t| t == "aa"));
        assert!(has_aa, "Expected 'aa' to be merged");
    }

    #[test]
    fn test_train_unicode() {
        let pretokens = ["你好", "你好", "世界"];
        let vocab = train(pretokens.into_iter(), 15);

        // Should handle unicode without panicking
        // 4 special + 你 + 好 + 世 + 界 = 8 base tokens
        assert!(vocab.len() >= 8);
    }
}
