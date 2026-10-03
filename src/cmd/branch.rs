use crate::{error::GitsResult, git, selector};

pub fn run() -> GitsResult<i32> {
    let branches = git::branches()?;
    let idx = selector::select(&branches, "select branch")?;
    println!("{}", branches[idx]);
    Ok(0)
}
