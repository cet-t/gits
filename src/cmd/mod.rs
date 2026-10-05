mod branch;
mod commit;
mod diff;
mod merge;
mod other;
mod rebase;
mod show;
mod switch;

use crate::{cliargs::Command, cmd, selector};

pub fn run(command: Command, key_codes: &selector::KeyCodes) -> crate::error::GitsResult<i32> {
    match command {
        Command::Show { args } => cmd::show::run(&args, key_codes),
        Command::Diff { print, base, args } => cmd::diff::run(print, base, &args, key_codes),
        Command::Switch { args } => cmd::switch::run(&args, key_codes),
        Command::Merge { args } => cmd::merge::run(&args, key_codes),
        Command::Rebase { args } => cmd::rebase::run(&args, key_codes),
        Command::Commit => cmd::commit::run(),
        Command::Branch => cmd::branch::run(),
        Command::Other(args) => cmd::other::run(&args),
    }
}
