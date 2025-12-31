//! bpe-train command: Train BPE tokenizer from corpus

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
