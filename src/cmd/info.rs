
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
