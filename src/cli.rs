//! Command-line interface parsing and dispatch

/// Available subcommands
#[derive(Debug)]
pub enum Command {
    Train,
    Embed,
    Similar,
    Analogy,
    BpeTrain,
    BpeEncode,
    Info,
    Help,
}
