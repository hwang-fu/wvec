//! train command: Train word vectors on a corpus

use crate::bpe::{encode, load as load_vocab, save as save_vocab, train as train_bpe};
use crate::cli::{get_arg, has_flag};
use crate::ffi::{
    self, wvec_checkpoint_save, wvec_model_free, wvec_model_init, wvec_shutdown_reset,
    wvec_train_corpus,
};
use crate::input::text::TextReader;
use crate::text::normalize::normalize;
use crate::text::pretokenize::pretokenize;
use std::ffi::c_int;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    if has_flag(args, "--help") || has_flag(args, "-h") {
        print_help();
        return Ok(());
    }

    // Parse arguments
    let input = get_arg(args, "--input").ok_or("Missing --input <file>")?;
    let output = get_arg(args, "--output").ok_or("Missing --output <file>")?;
    let vocab_file = get_arg(args, "--vocab");
    let vocab_size: usize = get_arg(args, "--vocab-size")
        .unwrap_or_else(|| "50000".to_string())
        .parse()
        .map_err(|_| "Invalid --vocab-size")?;
    let dim: i32 = get_arg(args, "--dim")
        .unwrap_or_else(|| "100".to_string())
        .parse()
        .map_err(|_| "Invalid --dim")?;
    let window: i32 = get_arg(args, "--window")
        .unwrap_or_else(|| "5".to_string())
        .parse()
        .map_err(|_| "Invalid --window")?;
    let neg_samples: i32 = get_arg(args, "--neg-samples")
        .unwrap_or_else(|| "5".to_string())
        .parse()
        .map_err(|_| "Invalid --neg-samples")?;
    let lr: f32 = get_arg(args, "--lr")
        .unwrap_or_else(|| "0.025".to_string())
        .parse()
        .map_err(|_| "Invalid --lr")?;
    let epochs: usize = get_arg(args, "--epochs")
        .unwrap_or_else(|| "5".to_string())
        .parse()
        .map_err(|_| "Invalid --epochs")?;

    eprintln!("Training word vectors...");
    eprintln!("  Input: {}", input);
    eprintln!("  Output: {}", output);
    eprintln!(
        "  Dim: {}, Window: {}, Neg: {}, LR: {}, Epochs: {}",
        dim, window, neg_samples, lr, epochs
    );

    // Step 1: Read and preprocess corpus
    eprintln!("\n[1/5] Reading corpus...");
    let reader = TextReader::open(&input).map_err(|e| format!("Cannot open {}: {}", input, e))?;

    let mut pretokens: Vec<String> = Vec::new();
    for line_result in reader {
        let line = line_result.map_err(|e| format!("Read error: {}", e))?;
        let normalized = normalize(&line);
        for pt in pretokenize(&normalized) {
            pretokens.push(pt.text.to_string());
        }
    }
    eprintln!("  {} pre-tokens", pretokens.len());

    // Step 2: Train or load BPE vocabulary
    eprintln!("\n[2/5] Preparing vocabulary...");
    let vocab = if let Some(ref vf) = vocab_file {
        eprintln!("  Loading from {}", vf);
        load_vocab(Path::new(vf)).map_err(|e| format!("Cannot load vocab: {}", e))?
    } else {
        eprintln!("  Training BPE (target size: {})", vocab_size);
        let v = train_bpe(pretokens.iter().map(|s| s.as_str()), vocab_size);
        // Save vocab alongside model
        let vocab_path = format!("{}.vocab", output);
        save_vocab(&v, Path::new(&vocab_path)).map_err(|e| format!("Cannot save vocab: {}", e))?;
        eprintln!("  Saved vocabulary to {}", vocab_path);
        v
    };
    eprintln!("  Vocabulary size: {}", vocab.len());

    // Step 3: Encode corpus to token IDs
    eprintln!("\n[3/5] Encoding corpus...");
    let mut token_ids: Vec<c_int> = Vec::new();
    for pt in &pretokens {
        let ids = encode(&vocab, pt);
        token_ids.extend(ids.iter().map(|&id| id as c_int));
    }
    eprintln!("  {} token IDs", token_ids.len());

    // Step 4: Build negative sampling table (unigram distribution)
    eprintln!("\n[4/5] Building negative sampling table...");
    let neg_table = build_neg_table(&token_ids, vocab.len());
    eprintln!("  Table size: {}", neg_table.len());

    // Step 5: Train model
    eprintln!("\n[5/5] Training...");
    unsafe {
        wvec_shutdown_reset();

        let status = wvec_model_init(vocab.len() as c_int, dim);
        if status != ffi::status::SUCCESS {
            return Err(format!("Failed to initialize model: {}", status));
        }

        for epoch in 1..=epochs {
            eprintln!("  Epoch {}/{}", epoch, epochs);

            let status = wvec_train_corpus(
                token_ids.as_ptr(),
                token_ids.len() as c_int,
                window,
                neg_samples,
                neg_table.as_ptr(),
                neg_table.len() as c_int,
                lr,
            );

            if status == ffi::status::STATUS_INTERRUPTED {
                eprintln!("  Interrupted! Saving checkpoint...");
                break;
            } else if status != ffi::status::SUCCESS {
                wvec_model_free();
                return Err(format!("Training failed: {}", status));
            }
        }

        // Save checkpoint
        let status = wvec_checkpoint_save(
            output.as_ptr() as *const i8,
            output.len() as c_int,
            epochs as c_int,
            lr,
        );
        if status != ffi::status::SUCCESS {
            wvec_model_free();
            return Err(format!("Failed to save model: {}", status));
        }

        wvec_model_free();
    }

    eprintln!("\nDone! Model saved to {}", output);
    Ok(())
}

/// Build negative sampling table from token frequencies
fn build_neg_table(token_ids: &[c_int], vocab_size: usize) -> Vec<c_int> {
    // Count token frequencies
    let mut counts = vec![0u64; vocab_size];
    for &id in token_ids {
        if (id as usize) < vocab_size {
            counts[id as usize] += 1;
        }
    }

    // Apply 3/4 power (reduces impact of very frequent words)
    let powered: Vec<f64> = counts.iter().map(|&c| (c as f64).powf(0.75)).collect();
    let total: f64 = powered.iter().sum();

    // Build table (size = 1M for good sampling resolution)
    const TABLE_SIZE: usize = 1_000_000;
    let mut table = Vec::with_capacity(TABLE_SIZE);

    let mut cumulative = 0.0;
    let mut word_idx = 0;

    for i in 0..TABLE_SIZE {
        let threshold = (i as f64 / TABLE_SIZE as f64) * total;
        while cumulative < threshold && word_idx < vocab_size {
            cumulative += powered[word_idx];
            word_idx += 1;
        }
        table.push((word_idx.saturating_sub(1)) as c_int);
    }

    table
}

fn print_help() {
    println!(
        "wvec train - Train word vectors

  USAGE:
      wvec train --input <file> --output <file> [OPTIONS]

  OPTIONS:
      --input <file>       Input text file
      --output <file>      Output model file (.bin)
      --vocab <file>       Load existing BPE vocabulary (optional)
      --vocab-size <n>     BPE vocabulary size (default: 50000)
      --dim <n>            Embedding dimension (default: 100)
      --window <n>         Context window size (default: 5)
      --neg-samples <n>    Negative samples (default: 5)
      --lr <f>             Learning rate (default: 0.025)
      --epochs <n>         Training epochs (default: 5)
      -h, --help           Show this help message"
    );
}
