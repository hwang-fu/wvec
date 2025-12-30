//! BPE Vocabulary
//!
//! Bidirectional mapping between tokens (strings) and IDs.

use std::collections::HashMap;

use crate::bpe::types::{BpePair, BpeTokenId};

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
}

impl Default for Vocabulary {
    fn default() -> Self {
        Self::new()
    }
}
