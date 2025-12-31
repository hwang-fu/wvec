
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
