//! wvec CLI entry point

use wvec::cli::{self, Args, SubCommand};
fn main() {
    let args = Args::parse();

    match args.cmd {
        SubCommand::Train => {
            println!("TODO: train command");
        }
        SubCommand::Embed => {
            println!("TODO: embed command");
        }
        SubCommand::Similar => {
            println!("TODO: similar command");
        }
        SubCommand::Analogy => {
            println!("TODO: analogy command");
        }
        SubCommand::BpeTrain => {
            println!("TODO: bpe-train command");
        }
        SubCommand::BpeEncode => {
            println!("TODO: bpe-encode command");
        }
        SubCommand::Info => {
            println!("TODO: info command");
        }
        SubCommand::Help => {
            cli::print_help();
        }
    }
}
