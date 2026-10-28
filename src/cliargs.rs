#[derive(::clap::Parser)]
#[command(
    name = "gits",
    version,
    about,
    disable_help_subcommand = true,
    arg_required_else_help = true,
    override_usage = "gits <command> [options]"
)]
pub struct Cli {
    /// Print the path of the config file and exit
    #[arg(long)]
    pub config_path: bool,

    /// Parse the config file, print it with how each entry is applied, and exit
    #[arg(long, conflicts_with = "config_path")]
    pub config: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

impl Cli {
    pub fn command(&self) -> Option<self::Command> {
        self.command.clone()
    }
}

#[derive(Clone, ::clap::Subcommand)]
pub enum Command {
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

    /// Any other command is passthrough to git as-is
    #[command(external_subcommand)]
    Other(Vec<String>),
}
