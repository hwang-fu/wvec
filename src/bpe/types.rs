//! BPE type definitions

/// A token ID (index into vocabulary)
pub type TokenId = u32;

/// Special token constants
pub const UNK_TOKEN: &str = "[UNK]";
pub const PAD_TOKEN: &str = "[PAD]";
pub const BOS_TOKEN: &str = "[BOS]";
pub const EOS_TOKEN: &str = "[EOS]";
