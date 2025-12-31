//! bpe-train command: Train BPE tokenizer from corpus

use crate::bpe::{save, train};
use crate::cli::{get_arg, has_flag};
use crate::input::text::TextReader;
use crate::text::normalize::normalize;
use crate::text::pretokenize::pretokenize;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    // Parse arguments
    if has_flag(args, "--help") || has_flag(args, "-h") {
        print_help();
        return Ok(());
    }

    let input = get_arg(args, "--input").ok_or("Missing --input <file>")?;
    let output = get_arg(args, "--output").ok_or("Missing --output <file>")?;
    let vocab_size: usize = get_arg(args, "--vocab-size")
        .unwrap_or_else(|| "10000".to_string())
        .parse()
        .map_err(|_| "Invalid --vocab-size")?;

    eprintln!("Training BPE tokenizer...");
    eprintln!("  Input: {}", input);
    eprintln!("  Output: {}", output);
    eprintln!("  Vocab size: {}", vocab_size);

    // Read and preprocess input
    let reader = TextReader::open(&input).map_err(|e| format!("Cannot open {}: {}", input, e))?;

    // Collect pre-tokens
    let mut pretokens: Vec<String> = Vec::new();
    for line_result in reader {
        let line = line_result.map_err(|e| format!("Read error: {}", e))?;
        let normalized = normalize(&line);
        for pt in pretokenize(&normalized) {
            pretokens.push(pt.text.to_string());
        }
    }

    eprintln!("  Collected {} pre-tokens", pretokens.len());

    // Train BPE
    let vocab = train(pretokens.iter().map(|s| s.as_str()), vocab_size);
    eprintln!("  Vocabulary: {} tokens", vocab.len());

    // Save vocabulary
    save(&vocab, Path::new(&output)).map_err(|e| format!("Cannot save vocab: {}", e))?;

    eprintln!("Done! Saved to {}", output);
    Ok(())
}

fn print_help() {
    println!(
        "wvec bpe-train - Train BPE tokenizer

  USAGE:
      wvec bpe-train --input <file> --output <file> [OPTIONS]

  OPTIONS:
      --input <file>       Input text file
      --output <file>      Output vocabulary file (.bin)
      --vocab-size <n>     Target vocabulary size (default: 10000)
      -h, --help           Show this help message"
    );
}
