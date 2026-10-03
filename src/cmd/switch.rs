use crate::{error::GitsResult, git, selector};

pub fn run(extra: &[String], key_codes: &selector::KeyCodes) -> GitsResult<i32> {
    let branches = git::branches()?;
    let idx = selector::select_with_keymap(&branches, "select branch", key_codes)?;
    let branch = branches[idx].clone();

    let mut args = vec!["switch".to_owned(), branch];
    args.extend_from_slice(extra);

    let status = git::execute(&args)?;
    Ok(status.code().unwrap_or(1))
}
