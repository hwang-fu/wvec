//! wvec CLI entry point

use wvec::cli::{self, Args, SubCommand};
fn main() {
    let args = Args::parse();

    match args.cmd {
        SubCommand::Train => {
            if let Err(e) = wvec::cmd::train::run(&args.args) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        SubCommand::Embed => {
            if let Err(e) = wvec::cmd::embed::run(&args.args) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        SubCommand::Similar => {
            if let Err(e) = wvec::cmd::similar::run(&args.args) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        SubCommand::Analogy => {
            if let Err(e) = wvec::cmd::analogy::run(&args.args) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        SubCommand::BpeTrain => {
            if let Err(e) = wvec::cmd::bpe_train::run(&args.args) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        SubCommand::BpeEncode => {
            if let Err(e) = wvec::cmd::bpe_encode::run(&args.args) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        SubCommand::Info => {
            if let Err(e) = wvec::cmd::info::run(&args.args) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        SubCommand::Help => {
            cli::print_help();
        }
    }
}
