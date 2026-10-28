use crate::{error::GitsResult, git, selector};

pub fn run(key_codes: &selector::KeyCodes) -> GitsResult<i32> {
    let lines = git::log_lines()?;
    let idx = selector::select_with_keymap(&lines, "select commit", key_codes)?;
    let hash = git::hash_from_line(&lines[idx]);
    println!("{hash}");
    Ok(0)
}
