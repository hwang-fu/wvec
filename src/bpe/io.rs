//! BPE Vocabulary File I/O
//!
//! Save and load trained BPE vocabularies to/from binary files.
//!
//! # File Format (v1)
//!
//! All integers are little-endian.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                        HEADER                           │
//! ├──────────────┬──────────┬───────────────────────────────┤
//! │ magic        │ [u8; 4]  │ "BPE\0" - file identifier     │
//! │ version      │ u32      │ format version (currently 1)  │
//! │ vocab_size   │ u32      │ number of tokens              │
//! │ pairs_count  │ u32      │ number of merge rules         │
//! ├──────────────┴──────────┴───────────────────────────────┤
//! │                    TOKENS SECTION                       │
//! │  Repeated `vocab_size` times, in ID order (0, 1, 2...)  │
//! ├──────────────┬──────────┬───────────────────────────────┤
//! │ len          │ u32      │ byte length of token string   │
//! │ bytes        │ [u8;len] │ UTF-8 encoded token           │
//! ├──────────────┴──────────┴───────────────────────────────┤
//! │                  MERGE PAIRS SECTION                    │
//! │  Repeated `pairs_count` times, in priority order        │
//! ├──────────────┬──────────┬───────────────────────────────┤
//! │ left         │ u32      │ left token ID of the pair     │
//! │ right        │ u32      │ right token ID of the pair    │
//! │ id           │ u32      │ merged token ID               │
//! └──────────────┴──────────┴───────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```text
//! // Save after training
//! let vocab = bpe::train(pretokens, 10000);
//! bpe::save(&vocab, Path::new("vocab.bin"))?;
//!
//! // Load for encoding
//! let vocab = bpe::load(Path::new("vocab.bin"))?;
//! let ids = bpe::encode(&vocab, "hello");
//! ```

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
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Write header
    writer.write_all(MAGIC)?;
    write_u32(&mut writer, VERSION)?;
    write_u32(&mut writer, vocab.len() as u32)?;
    write_u32(&mut writer, vocab.pairs_count() as u32)?;

    // Write tokens in ID order (0, 1, 2, ...)
    for id in 0..vocab.len() as u32 {
        let token = vocab.get_token(id).unwrap_or("");
        write_string(&mut writer, token)?;
    }

    // Write merge pairs
    for pair in vocab.pairs() {
        write_u32(&mut writer, pair.left)?;
        write_u32(&mut writer, pair.right)?;
        write_u32(&mut writer, pair.id)?;
    }

    writer.flush()
}

/// Loads a vocabulary from a binary file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or has invalid format.
pub fn load(path: &Path) -> io::Result<Vocabulary> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Read and verify header
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid magic bytes",
        ));
    }

    let version = read_u32(&mut reader)?;
    if version != VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported version: {}", version),
        ));
    }

    let vocab_size = read_u32(&mut reader)?;
    let pairs_count = read_u32(&mut reader)?;

    // Read tokens and build vocabulary
    let mut vocab = Vocabulary::empty(); // We need this method!
    for _id in 0..vocab_size {
        let token = read_string(&mut reader)?;
        vocab.add_token(token);
    }

    // Read merge pairs
    for _ in 0..pairs_count {
        let left = read_u32(&mut reader)?;
        let right = read_u32(&mut reader)?;
        let id = read_u32(&mut reader)?;
        vocab.add_pair(left, right, id);
    }

    Ok(vocab)
}

/// Writes a u32 in little-endian format.
fn write_u32<W: Write>(writer: &mut W, value: u32) -> io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

/// Writes a length-prefixed UTF-8 string.
fn write_string<W: Write>(writer: &mut W, s: &str) -> io::Result<()> {
    let bytes = s.as_bytes();
    write_u32(writer, bytes.len() as u32)?;
    writer.write_all(bytes)
}

/// Reads a u32 in little-endian format.
fn read_u32<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

/// Reads a length-prefixed UTF-8 string.
fn read_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let len = read_u32(reader)? as usize;

    // Prevent DoS: limit maximum token length
    const MAX_TOKEN_LEN: usize = 8 * 1024 * 1024; // 8MB
    if len > MAX_TOKEN_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("token too long: {} bytes", len),
        ));
    }

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use crate::bpe::train::train;

    #[test]
    fn test_save_load_roundtrip() {
        // Train a vocabulary
        let pretokens = ["hello", "hello", "world", "world", "world"];
        let vocab = train(pretokens.into_iter(), 20);

        // Save to temp file
        let path = Path::new("/tmp/test_vocab.bin");
        save(&vocab, path).expect("save failed");

        // Load it back
        let loaded = load(path).expect("load failed");

        // Verify
        assert_eq!(vocab.len(), loaded.len());
        assert_eq!(vocab.pairs_count(), loaded.pairs_count());

        // Check all tokens match
        for id in 0..vocab.len() as u32 {
            assert_eq!(vocab.get_token(id), loaded.get_token(id));
        }

        // Check all pairs match
        for (orig, loaded_pair) in vocab.pairs().iter().zip(loaded.pairs().iter()) {
            assert_eq!(orig.left, loaded_pair.left);
            assert_eq!(orig.right, loaded_pair.right);
            assert_eq!(orig.id, loaded_pair.id);
        }

        // Cleanup
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_load_invalid_magic() {
        let path = Path::new("/tmp/test_bad_magic.bin");
        fs::write(path, b"XXXX").expect("write failed");

        let result = load(path);
        assert!(result.is_err());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_save_load_empty_vocab() {
        let vocab = Vocabulary::new(); // Just special tokens

        let path = Path::new("/tmp/test_empty_vocab.bin");
        save(&vocab, path).expect("save failed");

        let loaded = load(path).expect("load failed");
        assert_eq!(vocab.len(), loaded.len());

        let _ = fs::remove_file(path);
    }
}
