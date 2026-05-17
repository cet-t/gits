use crate::{error::Error, git, selector};

pub fn run() -> Result<i32, Error> {
    let lines = git::log_lines()?;
    let idx = selector::select(&lines, "select commit")?;
    let hash = git::hash_from_line(&lines[idx]).to_owned();
    println!("{hash}");
    Ok(0)
}
