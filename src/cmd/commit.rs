use crate::{error::GitsResult, git, selector};

pub fn run() -> GitsResult<i32> {
    let lines = git::log_lines()?;
    let idx = selector::select(&lines, "select commit")?;
    let hash = git::hash_from_line(&lines[idx]).to_owned();
    println!("{hash}");
    Ok(0)
}
