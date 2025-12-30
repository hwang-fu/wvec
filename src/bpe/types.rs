//! BPE type definitions

/// A token ID (index into vocabulary)
pub type TokenId = u32;

/// Special token constants
pub const UNK_TOKEN: &str = "[UNK]";
pub const PAD_TOKEN: &str = "[PAD]";
pub const BOS_TOKEN: &str = "[BOS]";
pub const EOS_TOKEN: &str = "[EOS]";

/// Reserved IDs for special tokens
pub const UNK_ID: TokenId = 0;
pub const PAD_ID: TokenId = 1;
pub const BOS_ID: TokenId = 2;
pub const EOS_ID: TokenId = 3;
pub const FIRST_REGULAR_ID: TokenId = 4;

/// A merge rule: two tokens merge into one
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MergeRule {
    /// First token in the pair
    pub fst: TokenId,
    /// Second token in the pair
    pub snd: TokenId,
    /// Resulting merged token
    pub result: TokenId,
}

impl MergeRule {
    pub fn new(fst: TokenId, snd: TokenId, result: TokenId) -> Self {
        Self { fst, snd, result }
    }
}
