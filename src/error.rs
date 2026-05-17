use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("git failed (exit {code}): {stderr}")]
    GitFailed { code: i32, stderr: String },

    #[error("could not run git: {0}")]
    GitNotFound(#[source] std::io::Error),

    #[error("selection cancelled")]
    Cancelled,

    #[error("nothing to select")]
    Empty,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
