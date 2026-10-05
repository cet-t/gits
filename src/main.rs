mod cliargs;
mod cmd;
mod config;
mod error;
mod git;
mod selector;

use clap::Parser;

use crate::{cliargs::Cli, error::GitsError};

fn main() {
    let cli = Cli::parse();
    let key_codes = config::Config::load()
        .ok()
        .and_then(|c| c.keymap)
        .map(|km| selector::KeyCodes::from_config(&km))
        .unwrap_or_default();

    match cmd::run(cli.command(), &key_codes) {
        Ok(code) => std::process::exit(code),
        Err(GitsError::Cancelled) => std::process::exit(130),
        Err(e) => {
            eprintln!("gits: {e}");
            std::process::exit(1);
        }
    }
}
