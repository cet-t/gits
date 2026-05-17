use crate::{error::Error, git, selector};

pub fn run(extra: &[String]) -> Result<i32, Error> {
    let branches = git::branches()?;
    let idx = selector::select(&branches, "select branch to merge")?;
    let branch = branches[idx].clone();

    let mut args = vec!["merge".to_owned(), branch];
    args.extend_from_slice(extra);

    let status = git::exec(&args)?;
    Ok(status.code().unwrap_or(1))
}
