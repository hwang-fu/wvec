fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

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
