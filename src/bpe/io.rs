//! BPE Vocabulary File I/O
//!
//! Save and load trained BPE vocabularies to/from binary files.
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

use crate::bpe::vocab::Vocabulary;

/// Magic bytes identifying a BPE vocabulary file
const MAGIC: &[u8; 4] = b"BPE\0";

/// Current file format version
const VERSION: u32 = 1;

/// Saves a vocabulary to a binary file.
///
/// # Errors
///
/// Returns an error if the file cannot be created or written.
pub fn save(vocab: &Vocabulary, path: &Path) -> io::Result<()> {
    // TODO: Implement
    panic!("TODO: save")
}

/// Loads a vocabulary from a binary file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or has invalid format.
pub fn load(path: &Path) -> io::Result<Vocabulary> {
    // TODO: Implement
    panic!("TODO: load")
}

/// Writes a u32 in little-endian format.
fn write_u32<W: Write>(writer: &mut W, value: u32) -> io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load_roundtrip() {
        // TODO: Add tests
    }
}
