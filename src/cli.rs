//! Command-line interface parsing and dispatch

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
