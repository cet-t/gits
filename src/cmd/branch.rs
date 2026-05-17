use crate::{error::Error, git, selector};

pub fn run() -> Result<i32, Error> {
    let branches = git::branches()?;
    let idx = selector::select(&branches, "select branch")?;
    println!("{}", branches[idx]);
    Ok(0)
}
