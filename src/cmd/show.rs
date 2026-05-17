use crate::{error::Error, git, selector};

pub fn run(extra: &[String]) -> Result<i32, Error> {
    let lines = git::log_lines()?;
    let idx = selector::select(&lines, "select commit")?;
    let hash = git::hash_from_line(&lines[idx]).to_owned();

    let mut args = vec!["show".to_owned(), hash];
    args.extend_from_slice(extra);

    let status = git::exec(&args)?;
    Ok(status.code().unwrap_or(1))
}
