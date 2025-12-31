//! embed command: Get embedding vector for a word

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

    // Get embedding
    let mut embedding = vec![0.0f32; dim as usize];
    unsafe {
        let status = wvec_get_embedding(word_id as c_int, embedding.as_mut_ptr(), dim);
        if status != ffi::status::SUCCESS {
            wvec_model_free();
            return Err(format!("Cannot get embedding: {}", status));
        }
        wvec_model_free();
    }

    // Print embedding
    println!("Embedding for '{}' (dim={}):", word, dim);
    println!("[");
    for (i, val) in embedding.iter().enumerate() {
        if i > 0 && i % 8 == 0 {
            println!();
        }
        print!(" {:>9.6}", val);
    }
    println!("\n]");

    Ok(())
}

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
