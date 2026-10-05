use crate::{error::GitsResult, git};

pub fn run(args: &[String]) -> GitsResult<i32> {
    let status = git::execute(args)?;
    Ok(status.code().unwrap_or(-1))
}
