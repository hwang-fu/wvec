//! bpe-encode command: Encode text using BPE vocabulary

use crate::bpe::{encode, load as load_vocab};
use crate::cli::{get_arg, has_flag};
use crate::text::normalize::normalize;
use crate::text::pretokenize::pretokenize;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    if has_flag(args, "--help") || has_flag(args, "-h") {
        print_help();
        return Ok(());
    }

    let vocab_path = get_arg(args, "--vocab").ok_or("Missing --vocab <file>")?;
    let text = get_arg(args, "--text").ok_or("Missing --text <string>")?;

    // Load vocabulary
    let vocab =
        load_vocab(Path::new(&vocab_path)).map_err(|e| format!("Cannot load vocab: {}", e))?;

    // Normalize and pretokenize
    let normalized = normalize(&text);
    let pretokens: Vec<_> = pretokenize(&normalized);

    // Encode each pretoken
    println!("Input: {}", text);
    println!("Normalized: {}", normalized);
    println!("\nTokens:");

    let mut all_ids = Vec::new();
    for pt in pretokens {
        let ids = encode(&vocab, &pt.text);
        print!("  '{}' -> [", pt.text);
        for (i, &id) in ids.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", id);
        }
        println!("]");
        all_ids.extend(ids);
    }

    println!("\nAll token IDs: {:?}", all_ids);
    println!("Total: {} tokens", all_ids.len());

    Ok(())
}

fn print_help() {
    println!(
        "wvec bpe-encode - Encode text using BPE vocabulary

  USAGE:
      wvec bpe-encode --vocab <file> --text <string>

  OPTIONS:
      --vocab <file>       Path to BPE vocabulary file
      --text <string>      Text to encode
      -h, --help           Show this help message"
    );
}
