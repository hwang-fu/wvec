
/// Parse "king - man + woman" into [(king, 1.0), (man, -1.0), (woman, 1.0)]
fn parse_query(query: &str) -> Result<Vec<(String, f32)>, String> {
    let mut terms = Vec::new();
    let mut sign = 1.0f32;

    for part in query.split_whitespace() {
        match part {
            "+" => sign = 1.0,
            "-" => sign = -1.0,
            word => {
                terms.push((word.to_string(), sign));
                sign = 1.0; // Reset to positive for next word
            }
        }
    }

    Ok(terms)
}

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
