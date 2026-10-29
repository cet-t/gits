mod ansi;
mod cliargs;
mod cmd;
mod config;
mod error;
mod git;
mod selector;

use clap::{CommandFactory, Parser, error::ErrorKind};

use crate::{cliargs::Cli, error::GitsError};

fn main() {
    let cli = Cli::parse();

    if cli.config_path || cli.config {
        let flag = if cli.config {
            "--config"
        } else {
            "--config-path"
        };
        if cli.command().is_some() {
            <Cli as CommandFactory>::command()
                .error(
                    ErrorKind::ArgumentConflict,
                    format!("{flag} cannot be used with a command"),
                )
                .exit();
        }
        let result = if cli.config {
            config::describe()
        } else {
            config::config_path().map(|path| format!("{}\n", path.display()))
        };
        match result {
            Ok(text) => {
                print!("{text}");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("gits: {e}");
                std::process::exit(1);
            }
        }
    }

    let Some(command) = cli.command() else {
        let _ = <Cli as CommandFactory>::command().print_help();
        std::process::exit(2);
    };

    let key_codes = config::Config::load()
        .ok()
        .and_then(|c| c.keymap)
        .map(|km| selector::KeyCodes::from_config(&km))
        .unwrap_or_default();

    match cmd::run(command, &key_codes) {
        Ok(code) => std::process::exit(code),
        Err(GitsError::Cancelled) => std::process::exit(130),
        Err(e) => {
            eprintln!("gits: {e}");
            std::process::exit(1);
        }
    }
}
