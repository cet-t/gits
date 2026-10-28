use crate::{error::GitsResult, git, selector};

pub fn run(key_codes: &selector::KeyCodes) -> GitsResult<i32> {
    let branches = git::branches()?;
    let idx = selector::select_with_keymap(
        &git::colored_branches(&branches),
        "select branch",
        key_codes,
    )?;
    println!("{}", branches[idx]);
    Ok(0)
}
