//! similar command: Find similar words

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
    let word = get_arg(args, "--word").ok_or("Missing --word <word>")?;
    let topk: usize = get_arg(args, "--topk")
        .unwrap_or_else(|| "10".to_string())
        .parse()
        .map_err(|_| "Invalid --topk")?;

    // Derive vocab path from model path
    let vocab_path = format!("{}.vocab", model_path);

    // Load vocabulary
    let vocab = load_vocab(Path::new(&vocab_path))
        .map_err(|e| format!("Cannot load vocab {}: {}", vocab_path, e))?;

    // Find word ID
    let word_id = vocab
        .get_id_opt(&word)
        .ok_or(format!("Word '{}' not in vocabulary", word))?;

    // Load model
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

    // Get dimensions
    let mut vocab_size: c_int = 0;
    let mut dim: c_int = 0;
    unsafe {
        ffi::wvec_model_get_dims(&mut vocab_size, &mut dim);
    }

    // Get query embedding
    let mut query_emb = vec![0.0f32; dim as usize];
    unsafe {
        let status = wvec_get_embedding(word_id as c_int, query_emb.as_mut_ptr(), dim);
        if status != ffi::status::SUCCESS {
            wvec_model_free();
            return Err(format!("Cannot get embedding for '{}': {}", word, status));
        }
    }

    // Compute similarities with all words
    let mut similarities: Vec<(u32, f32)> = Vec::new();
    let mut other_emb = vec![0.0f32; dim as usize];

    for id in 0..vocab_size as u32 {
        if id == word_id {
            continue; // Skip the query word itself
        }

        unsafe {
            let status = wvec_get_embedding(id as c_int, other_emb.as_mut_ptr(), dim);
            if status != ffi::status::SUCCESS {
                continue;
            }
        }

        let sim = cosine_similarity(&query_emb, &other_emb);
        similarities.push((id, sim));
    }

    // Sort by similarity (descending)
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Print results
    println!("Similar to '{}':", word);
    for (id, sim) in similarities.iter().take(topk) {
        if let Some(token) = vocab.get_token(*id) {
            println!("  {:>6.4}  {}", sim, token);
        }
    }

    unsafe {
        wvec_model_free();
    }

    Ok(())
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
