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
}

impl Default for Vocabulary {
    fn default() -> Self {
        Self::new()
    }
}
