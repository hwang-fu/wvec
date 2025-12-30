//! BPE type definitions

/// A token ID (index into vocabulary)
pub type BpeTokenId = u32;

/// Unknown token, used when a word is not in the vocabulary
pub const UNK_TOKEN: &str = "[UNK]";

/// Padding token, used to fill sequences to equal length for batch processing
pub const PAD_TOKEN: &str = "[PAD]";

/// Beginning of Sequence marker, indicates where a sentence starts
pub const BOS_TOKEN: &str = "[BOS]";

/// End of Sequence marker, indicates where a sentence ends
pub const EOS_TOKEN: &str = "[EOS]";

/// Reserved IDs for special tokens
pub const UNK_ID: BpeTokenId = 0;
pub const PAD_ID: BpeTokenId = 1;
pub const BOS_ID: BpeTokenId = 2;
pub const EOS_ID: BpeTokenId = 3;
pub const FIRST_REGULAR_ID: BpeTokenId = 4;

/// A merge rule: two tokens merge into one
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BpePair {
    /// Left token in the pair
    pub left: BpeTokenId,
    /// Right token in the pair
    pub right: BpeTokenId,
    /// Resulting merged token id
    pub id: BpeTokenId,
}

impl BpePair {
    pub fn new(left: BpeTokenId, right: BpeTokenId, id: BpeTokenId) -> Self {
        Self { left, right, id }
    }

    pub fn pair(&self) -> (BpeTokenId, BpeTokenId) {
        (self.left, self.right)
    }

    pub fn id(&self) -> BpeTokenId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_token_ids() {
        assert_eq!(UNK_ID, 0);
        assert_eq!(PAD_ID, 1);
        assert_eq!(BOS_ID, 2);
        assert_eq!(EOS_ID, 3);
        assert_eq!(FIRST_REGULAR_ID, 4);
    }

    #[test]
    fn test_merge_rule() {
        let rule = BpePair::new(10, 20, 30);
        assert_eq!(rule.left, 10);
        assert_eq!(rule.right, 20);
        assert_eq!(rule.id, 30);
    }
}
