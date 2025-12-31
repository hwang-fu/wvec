
fn print_help() {
    println!(
        "wvec embed - Get embedding vector for a word

  USAGE:
      wvec embed --model <file> --word <word>

  OPTIONS:
      --model <file>       Path to trained model
      --word <word>        Word to get embedding for
      -h, --help           Show this help message"
    );
}
