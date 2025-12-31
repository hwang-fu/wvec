use crate::bpe::load;
use crate::cli::{get_arg, has_flag};
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    if has_flag(args, "--help") || has_flag(args, "-h") {
        print_help();
        return Ok(());
    }

    // Get file path (positional or --model/--vocab)
    let path = get_arg(args, "--model")
        .or_else(|| get_arg(args, "--vocab"))
        .or_else(|| args.first().cloned())
        .ok_or("Missing file path")?;

    // Detect file type by trying to load as vocab
    let path_ref = Path::new(&path);

    match load(path_ref) {
        Ok(vocab) => {
            println!("BPE Vocabulary: {}", path);
            println!("  Tokens: {}", vocab.len());
            println!("  Merge rules: {}", vocab.pairs().len());

            // Show sample tokens
            println!("\n  Sample tokens:");
            for id in 0..vocab.len().min(10) as u32 {
                if let Some(token) = vocab.get_token(id) {
                    let display = if token.contains('\n') || token.contains('\r') {
                        format!("{:?}", token) // Escape special chars
                    } else {
                        token.to_string()
                    };
                    println!("    {:>4}: {}", id, display);
                }
            }
        }
        Err(e) => {
            return Err(format!("Cannot load {}: {}", path, e));
        }
    }

    Ok(())
}

fn print_help() {
    println!(
        "wvec info - Show model/vocabulary information

  USAGE:
      wvec info <file>
      wvec info --vocab <file>
      wvec info --model <file>

  ARGUMENTS:
      <file>              Path to vocabulary (.bin) or model file

  OPTIONS:
      --vocab <file>      Path to BPE vocabulary file
      --model <file>      Path to trained model file
      -h, --help          Show this help message"
    );
}
