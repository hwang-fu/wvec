
fn print_help() {
    println!(
        "wvec similar - Find similar words

  USAGE:
      wvec similar --model <file> --word <word> [OPTIONS]

  OPTIONS:
      --model <file>       Path to trained model
      --word <word>        Query word
      --topk <n>           Number of results (default: 10)
      -h, --help           Show this help message"
    );
}
