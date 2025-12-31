
fn print_help() {
    println!(
        "wvec analogy - Solve word analogies

  USAGE:
      wvec analogy --model <file> --query <expr>

  EXAMPLES:
      wvec analogy --model model.bin --query \"king - man + woman\"

  OPTIONS:
      --model <file>       Path to trained model
      --query <expr>       Analogy expression (e.g., \"king - man + woman\")
      --topk <n>           Number of results (default: 5)
      -h, --help           Show this help message"
    );
}
