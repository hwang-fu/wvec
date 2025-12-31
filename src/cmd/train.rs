use std::ffi::c_int;

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
