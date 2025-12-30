//! BPE Vocabulary
//!
//! Bidirectional mapping between tokens (strings) and IDs.

use std::collections::HashMap;

use crate::bpe::types::{BOS_TOKEN, BpePair, BpeTokenId, EOS_TOKEN, PAD_TOKEN, UNK_ID, UNK_TOKEN};

/// BPE Vocabulary with bidirectional lookup
#[derive(Debug, Clone)]
pub struct Vocabulary {
    token_to_id: HashMap<String, BpeTokenId>,
    id_to_token: Vec<String>,
    pairs: Vec<BpePair>,
}

impl Vocabulary {
    pub fn new() -> Self {
        let mut vocab = Self {
            token_to_id: HashMap::new(),
            id_to_token: Vec::new(),
            pairs: Vec::new(),
        };

        // Register special tokens
        vocab.add_token(UNK_TOKEN.to_string());
        vocab.add_token(PAD_TOKEN.to_string());
        vocab.add_token(BOS_TOKEN.to_string());
        vocab.add_token(EOS_TOKEN.to_string());

        vocab
    }

    /// Returns vocabulary size (number of tokens).
    pub fn len(&self) -> usize {
        self.id_to_token.len()
    }

    /// Returns true if vocabulary is empty.
    pub fn is_empty(&self) -> bool {
        self.id_to_token.is_empty()
    }

    /// Adds a token and returns its ID. If already exists, returns existing ID.
    pub fn add_token(&mut self, token: String) -> BpeTokenId {
        if let Some(&id) = self.token_to_id.get(&token) {
            return id;
        }

        let id = self.id_to_token.len() as BpeTokenId;
        self.token_to_id.insert(token.clone(), id);
        self.id_to_token.push(token);
        id
    }

    pub fn get_id(&self, token: &str) -> BpeTokenId {
        self.token_to_id.get(token).copied().unwrap_or(UNK_ID)
    }

    pub fn get_id_opt(&self, token: &str) -> Option<BpeTokenId> {
        self.token_to_id.get(token).copied()
    }

    pub fn get_token(&self, id: BpeTokenId) -> Option<&str> {
        self.id_to_token.get(id as usize).map(|s| s.as_str())
    }

    pub fn add_pair(&mut self, left: BpeTokenId, right: BpeTokenId, id: BpeTokenId) {
        self.pairs.push(BpePair::new(left, right, id))
    }

    /// Returns BPE pairs in priority order.
    pub fn pairs(&self) -> &[BpePair] {
        &self.pairs
    }

    pub fn pairs_count(&self) -> usize {
        self.pairs.len()
    }

    pub fn contains(&self, token: &str) -> bool {
        self.token_to_id.contains_key(token)
    }

    /// Iterates over all (token, id) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, BpeTokenId)> {
        self.id_to_token
            .iter()
            .enumerate()
            .map(|(id, tk)| (tk.as_str(), id as BpeTokenId))
    }
}

impl Default for Vocabulary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::bpe::types::{BOS_ID, EOS_ID, FIRST_REGULAR_ID, PAD_ID};

    use super::*;

    #[test]
    fn test_new_has_special_tokens() {
        let vocab = Vocabulary::new();
        assert_eq!(vocab.len(), 4);
        assert_eq!(vocab.get_id(UNK_TOKEN), UNK_ID);
        assert_eq!(vocab.get_id(PAD_TOKEN), PAD_ID);
        assert_eq!(vocab.get_id(BOS_TOKEN), BOS_ID);
        assert_eq!(vocab.get_id(EOS_TOKEN), EOS_ID);
    }

    #[test]
    fn test_add_token() {
        let mut vocab = Vocabulary::new();
        let id = vocab.add_token("hello".to_string());
        assert_eq!(id, FIRST_REGULAR_ID);
        assert_eq!(vocab.get_id("hello"), id);
        assert_eq!(vocab.get_token(id), Some("hello"));
    }

    #[test]
    fn test_add_duplicate() {
        let mut vocab = Vocabulary::new();
        let id1 = vocab.add_token("hello".to_string());
        let id2 = vocab.add_token("hello".to_string());
        assert_eq!(id1, id2);
        assert_eq!(vocab.len(), 5); // 4 special + 1
    }

    #[test]
    fn test_unknown_token() {
        let vocab = Vocabulary::new();
        assert_eq!(vocab.get_id("nonexistent"), UNK_ID);
        assert_eq!(vocab.get_id_opt("nonexistent"), None);
    }

    #[test]
    fn test_pairs() {
        let mut vocab = Vocabulary::new();
        let a = vocab.add_token("a".to_string());
        let b = vocab.add_token("b".to_string());
        let ab = vocab.add_token("ab".to_string());

        vocab.add_pair(a, b, ab);

        assert_eq!(vocab.pairs_count(), 1);
        let pair = &vocab.pairs()[0];
        assert_eq!(pair.left, a);
        assert_eq!(pair.right, b);
        assert_eq!(pair.id, ab);
    }

    #[test]
    fn test_iter() {
        let mut vocab = Vocabulary::new();
        vocab.add_token("test".to_string());

        let pairs: Vec<_> = vocab.iter().collect();
        assert_eq!(pairs.len(), 5);
        assert!(pairs.contains(&(UNK_TOKEN, UNK_ID)));
        assert!(pairs.contains(&("test", FIRST_REGULAR_ID)));
    }

    #[test]
    fn test_contains() {
        let mut vocab = Vocabulary::new();
        vocab.add_token("exists".to_string());

        assert!(vocab.contains("exists"));
        assert!(vocab.contains(UNK_TOKEN));
        assert!(!vocab.contains("missing"));
    }
}
