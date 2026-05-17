use crate::{error::Error, git, selector};

pub fn run(print: bool, base: Option<&str>, extra: &[String]) -> Result<i32, Error> {
    let lines = git::log_lines()?;

    let base_hash = match base {
        Some(b) => b.to_owned(),
        None => {
            let idx = selector::select(&lines, "select base commit")?;
            git::hash_from_line(&lines[idx]).to_owned()
        }
    };

    let idx = selector::select(&lines, "select target commit")?;
    let target_hash = git::hash_from_line(&lines[idx]).to_owned();

    if print {
        println!("{base_hash} {target_hash}");
        return Ok(0);
    }

    let mut args = vec!["diff".to_owned(), base_hash, target_hash];
    args.extend_from_slice(extra);

    let status = git::exec(&args)?;
    Ok(status.code().unwrap_or(1))
}
