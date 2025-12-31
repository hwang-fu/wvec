
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
