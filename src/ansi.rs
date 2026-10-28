/// Remove ANSI CSI escape sequences (e.g. `\x1b[1;32m`) from `s`.
pub fn strip(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            // Parameters and intermediates run until a final byte in '@'..='~'.
            for c in chars.by_ref() {
                if ('@'..='~').contains(&c) {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::strip;

    #[test]
    fn strips_git_log_colors() {
        let line = "\x1b[33m52ec6c7\x1b[m\x1b[33m (\x1b[m\x1b[1;36mHEAD\x1b[m\x1b[33m -> \x1b[m\x1b[1;32mmain\x1b[m\x1b[33m)\x1b[m feat: x";
        assert_eq!(strip(line), "52ec6c7 (HEAD -> main) feat: x");
    }

    #[test]
    fn keeps_plain_text() {
        assert_eq!(strip("main"), "main");
    }
}
