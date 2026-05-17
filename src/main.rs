mod cmd;
mod error;
mod git;
mod selector;

use clap::{Parser, Subcommand};

use error::Error;

#[derive(Parser)]
#[command(
    name = "gits",
    about = "Git argument selector — interactive ref picker for git commands",
    disable_help_subcommand = true,
    override_usage = "gits <command> [options]"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Select a commit and run: git show <commit> [options]
    Show {
        /// Options passed directly to git show
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },

    /// Select two commits and run: git diff <base> <target> [options]
    ///
    /// Intended for commit-to-commit diffs only.
    /// All unknown options are forwarded to git diff.
    Diff {
        /// Print "<base> <target>" to stdout instead of running git diff
        #[arg(long)]
        print: bool,

        /// Skip base selection and use REF as the base commit
        #[arg(long, value_name = "REF")]
        base: Option<String>,

        /// Options passed directly to git diff
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },

    /// Select a branch and run: git switch <branch> [options]
    Switch {
        /// Options passed directly to git switch
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },

    /// Select a branch and run: git merge <branch> [options]
    Merge {
        /// Options passed directly to git merge
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },

    /// Select a branch and run: git rebase <branch> [options]
    Rebase {
        /// Options passed directly to git rebase
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },

    /// Select a commit and print its hash to stdout
    ///
    /// Usage:
    ///     git rebase -i $(gits commit)
    ///     git diff $(gits commit) HEAD
    Commit,

    /// Select a branch and print its name to stdout
    ///
    /// Usage:
    ///     git switch $(gits branch)
    Branch,
}

fn main() {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => std::process::exit(code),
        Err(Error::Cancelled) => std::process::exit(130),
        Err(e) => {
            eprintln!("gits: {e}");
            std::process::exit(1);
        }
    }
}

fn run(cli: Cli) -> Result<i32, Error> {
    match cli.command {
        Cmd::Show { args } => cmd::show::run(&args),
        Cmd::Diff { print, base, args } => cmd::diff::run(print, base.as_deref(), &args),
        Cmd::Switch { args } => cmd::switch::run(&args),
        Cmd::Merge { args } => cmd::merge::run(&args),
        Cmd::Rebase { args } => cmd::rebase::run(&args),
        Cmd::Commit => cmd::commit::run(),
        Cmd::Branch => cmd::branch::run(),
    }
}
