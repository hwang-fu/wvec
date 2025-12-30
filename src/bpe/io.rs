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
    // TODO: Implement
    panic!("TODO: load")
}

/// Writes a u32 in little-endian format.
fn write_u32<W: Write>(writer: &mut W, value: u32) -> io::Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    writer.flush()
}

/// Writes a length-prefixed UTF-8 string.
fn write_string<W: Write>(writer: &mut W, s: &str) -> io::Result<()> {
    let bytes = s.as_bytes();
    write_u32(writer, bytes.len() as u32)?;
    writer.write_all(bytes)?;
    writer.flush()
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
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load_roundtrip() {
        // TODO: Add tests
    }
}
