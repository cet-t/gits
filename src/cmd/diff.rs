use crate::{error::Error, git, selector};

pub fn run(print: bool, base: Option<&str>, extra: &[String], key_codes: &selector::KeyCodes) -> Result<i32, Error> {
    let lines = git::log_lines()?;

    let base_hash = match base {
        Some(b) => b.to_owned(),
        None => {
            let idx = selector::select_with_keymap(&lines, "select base commit", key_codes)?;
            git::hash_from_line(&lines[idx]).to_owned()
        }
    };

    let idx = selector::select_with_keymap(&lines, "select target commit", key_codes)?;
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
