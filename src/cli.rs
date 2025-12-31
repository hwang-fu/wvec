//! Command-line interface parsing and dispatch

use std::env;

/// Available subcommands
#[derive(Debug)]
pub enum SubCommand {
    Train,
    Embed,
    Similar,
    Analogy,
    BpeTrain,
    BpeEncode,
    Info,
    Help,
}

/// Parsed command-line arguments
#[derive(Debug)]
pub struct Args {
    pub cmd: SubCommand,
    pub args: Vec<String>,
}

impl Args {
    /// Parse command-line arguments
    pub fn parse() -> Self {
        let total_args: Vec<String> = env::args().collect();

        // Skip program name
        let cmd = total_args.get(1).cloned();
        let args: Vec<String> = total_args.into_iter().skip(2).collect();

        let cmd = match cmd.as_deref() {
            Some("train") => SubCommand::Train,
            Some("embed") => SubCommand::Embed,
            Some("similar") => SubCommand::Similar,
            Some("analogy") => SubCommand::Analogy,
            Some("bpe-train") => SubCommand::BpeTrain,
            Some("bpe-encode") => SubCommand::BpeEncode,
            Some("info") => SubCommand::Info,
            Some("help") | Some("--help") | Some("-h") => SubCommand::Help,
            Some(other) => {
                eprintln!("Unknown command: {}", other);
                SubCommand::Help
            }
            None => SubCommand::Help,
        };

        Self { cmd, args }
    }
}

/// Simple argument parser for --key value pairs
pub fn get_arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1).cloned())
}

/// Check if flag is present
pub fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

/// Print usage information
pub fn print_help() {
    println!(
        "wvec - Word2Vec in Rust + Fortran

  USAGE:
      wvec <COMMAND> [OPTIONS]

  COMMANDS:
      train       Train word vectors on a corpus
      embed       Get embedding for text
      similar     Find similar words
      analogy     Solve word analogies
      bpe-train   Train BPE tokenizer
      bpe-encode  Encode text with BPE
      info        Show model information
      help        Show this help message

  Run 'wvec <COMMAND> --help' for more information on a command."
    );
}
