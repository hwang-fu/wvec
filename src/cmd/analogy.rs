//! analogy command: Solve word analogies (king - man + woman = ?)

use crate::bpe::load as load_vocab;
use crate::cli::{get_arg, has_flag};
use crate::ffi::{self, wvec_checkpoint_load, wvec_get_embedding, wvec_model_free};
use std::ffi::c_int;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    if has_flag(args, "--help") || has_flag(args, "-h") {
        print_help();
        return Ok(());
    }

    let model_path = get_arg(args, "--model").ok_or("Missing --model <file>")?;
    let query = get_arg(args, "--query").ok_or("Missing --query <expr>")?;
    let topk: usize = get_arg(args, "--topk")
        .unwrap_or_else(|| "5".to_string())
        .parse()
        .map_err(|_| "Invalid --topk")?;

    // Parse query: "king - man + woman" -> [(king, +1), (man, -1), (woman, +1)]
    let terms = parse_query(&query)?;
    if terms.is_empty() {
        return Err("Empty query".to_string());
    }

    // Load vocab and model
    let vocab_path = format!("{}.vocab", model_path);
    let vocab =
        load_vocab(Path::new(&vocab_path)).map_err(|e| format!("Cannot load vocab: {}", e))?;

    let mut epoch: c_int = 0;
    let mut lr: f32 = 0.0;
    unsafe {
        let status = wvec_checkpoint_load(
            model_path.as_ptr() as *const i8,
            model_path.len() as c_int,
            &mut epoch,
            &mut lr,
        );
        if status != ffi::status::SUCCESS {
            return Err(format!("Cannot load model: {}", status));
        }
    }

    let mut vocab_size: c_int = 0;
    let mut dim: c_int = 0;
    unsafe {
        ffi::wvec_model_get_dims(&mut vocab_size, &mut dim);
    }

    // Build query vector
    let mut query_vec = vec![0.0f32; dim as usize];
    let mut query_word_ids: Vec<u32> = Vec::new();

    for (word, sign) in &terms {
        let word_id = vocab
            .get_id_opt(word)
            .ok_or(format!("Word '{}' not in vocabulary", word))?;
        query_word_ids.push(word_id);

        let mut emb = vec![0.0f32; dim as usize];
        unsafe {
            let status = wvec_get_embedding(word_id as c_int, emb.as_mut_ptr(), dim);
            if status != ffi::status::SUCCESS {
                wvec_model_free();
                return Err(format!("Cannot get embedding for '{}': {}", word, status));
            }
        }

        for (q, e) in query_vec.iter_mut().zip(emb.iter()) {
            *q += sign * e;
        }
    }

    // Find most similar (excluding query words)
    let mut similarities: Vec<(u32, f32)> = Vec::new();
    let mut other_emb = vec![0.0f32; dim as usize];

    for id in 0..vocab_size as u32 {
        if query_word_ids.contains(&id) {
            continue;
        }

        unsafe {
            let status = wvec_get_embedding(id as c_int, other_emb.as_mut_ptr(), dim);
            if status != ffi::status::SUCCESS {
                continue;
            }
        }

        let sim = cosine_similarity(&query_vec, &other_emb);
        similarities.push((id, sim));
    }

    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    unsafe {
        wvec_model_free();
    }

    // Print results
    println!("Analogy: {}", query);
    println!("Results:");
    for (id, sim) in similarities.iter().take(topk) {
        if let Some(token) = vocab.get_token(*id) {
            println!("  {:>6.4}  {}", sim, token);
        }
    }

    Ok(())
}

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
