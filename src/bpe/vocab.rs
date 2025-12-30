//! BPE Vocabulary
//!
//! Bidirectional mapping between tokens (strings) and IDs.

use std::collections::HashMap;

use crate::bpe::types::{MergeRule, TokenId};

/// BPE Vocabulary with bidirectional lookup
#[derive(Debug, Clone)]
pub struct Vocabulary {
    token_to_id: HashMap<String, TokenId>,
    id_to_token: Vec<String>,
    merges: Vec<MergeRule>,
}
