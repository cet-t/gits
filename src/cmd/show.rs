use crate::{error::GitsResult, git, selector};

pub fn run(extra: &[String], key_codes: &selector::KeyCodes) -> GitsResult<i32> {
    let lines = git::log_lines()?;
    let idx = selector::select_with_keymap(&lines, "select commit", key_codes)?;
    let hash = git::hash_from_line(&lines[idx]).to_owned();

    let mut args = vec!["show".to_owned(), hash];
    args.extend_from_slice(extra);

    let status = git::execute(&args)?;
    Ok(status.code().unwrap_or(1))
}
