use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::config;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct KeyCodes {
    pub up_keys: Vec<(KeyCode, KeyModifiers)>,
    pub down_keys: Vec<(KeyCode, KeyModifiers)>,
    pub confirm_keys: Vec<(KeyCode, KeyModifiers)>,
    pub cancel_keys: Vec<(KeyCode, KeyModifiers)>,
}

impl KeyCodes {
    pub fn from_config(keymap: &config::Keymap) -> Self {
        let parse = |keys: &[String]| {
            keys.iter()
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .filter_map(|s| parse_key(&s))
                .collect::<Vec<_>>()
        };
        Self {
            up_keys: parse(&keymap.up),
            down_keys: parse(&keymap.down),
            confirm_keys: parse(&keymap.confirm),
            cancel_keys: parse(&keymap.cancel),
        }
    }
}

impl Default for KeyCodes {
    fn default() -> Self {
        Self {
            up_keys: vec![],
            down_keys: vec![],
            confirm_keys: vec![],
            cancel_keys: vec![],
        }
    }
}

pub fn select(items: &[String], prompt: &str) -> Result<usize, Error> {
    select_with_keymap(items, prompt, &KeyCodes::default())
}

pub fn select_with_keymap(items: &[String], prompt: &str, key_codes: &KeyCodes) -> Result<usize, Error> {
    if items.is_empty() {
        return Err(Error::Empty);
    }

    let stderr = io::stderr();
    let mut out = stderr.lock();

    terminal::enable_raw_mode()?;
    let result = run_select(&mut out, items, prompt, key_codes);

    let _ = execute!(out, cursor::Show);
    terminal::disable_raw_mode()?;

    result
}

fn run_select(
    out: &mut impl Write,
    items: &[String],
    prompt: &str,
    key_codes: &KeyCodes,
) -> Result<usize, Error> {
    let (_, rows) = terminal::size()?;
    let mut visible = (rows as usize).saturating_sub(2).max(1).min(items.len());

    let mut sel = 0usize;
    let mut off = 0usize;

    while event::poll(std::time::Duration::ZERO)? {
        event::read()?;
    }

    execute!(out, cursor::Hide)?;
    render(out, items, sel, off, visible, prompt)?;

    drain_events();

    loop {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let matched = (key.code, key.modifiers);

                let is_up = bindings_contain(&key_codes.up_keys, matched)
                    || (key_codes.up_keys.is_empty()
                        && matches!(matched, (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE)));

                let is_down = bindings_contain(&key_codes.down_keys, matched)
                    || (key_codes.down_keys.is_empty()
                        && matches!(matched, (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE)));

                let is_confirm = bindings_contain(&key_codes.confirm_keys, matched)
                    || (key_codes.confirm_keys.is_empty()
                        && matches!(matched, (KeyCode::Enter, _) | (KeyCode::Char('l'), KeyModifiers::NONE)));

                let is_cancel = bindings_contain(&key_codes.cancel_keys, matched)
                    || (key_codes.cancel_keys.is_empty()
                        && matches!(
                            matched,
                            (KeyCode::Esc, _)
                                | (KeyCode::Char('h'), KeyModifiers::NONE)
                                | (KeyCode::Char('q'), KeyModifiers::NONE)
                                | (KeyCode::Char('c'), KeyModifiers::CONTROL)
                        ));

                if is_up {
                    if sel > 0 {
                        sel -= 1;
                        if sel < off {
                            off = sel;
                        }
                        erase(out, visible + 1)?;
                        render(out, items, sel, off, visible, prompt)?;
                    }
                } else if is_down {
                    if sel + 1 < items.len() {
                        sel += 1;
                        if sel >= off + visible {
                            off = sel + 1 - visible;
                        }
                        erase(out, visible + 1)?;
                        render(out, items, sel, off, visible, prompt)?;
                    }
                } else if is_confirm {
                    erase(out, visible + 1)?;
                    return Ok(sel);
                } else if is_cancel {
                    erase(out, visible + 1)?;
                    return Err(Error::Cancelled);
                }
            }
            Event::Resize(_, new_rows) => {
                let new_visible = (new_rows as usize).saturating_sub(2).max(1).min(items.len());
                erase(out, visible + 1)?;
                visible = new_visible;
                if sel >= off + visible {
                    off = sel + 1 - visible;
                }
                render(out, items, sel, off, visible, prompt)?;
            }
            _ => {}
        }
    }
}

fn drain_events() {
    while event::poll(std::time::Duration::ZERO).ok().unwrap_or(false) {
        let _ = event::read();
    }
}

fn bindings_contain(bindings: &[(KeyCode, KeyModifiers)], target: (KeyCode, KeyModifiers)) -> bool {
    bindings.iter().any(|&b| b == target)
}

fn render(
    out: &mut impl Write,
    items: &[String],
    sel: usize,
    off: usize,
    visible: usize,
    prompt: &str,
) -> Result<(), Error> {
    queue!(
        out,
        SetForegroundColor(Color::DarkYellow),
        Print(format!("  {prompt}\n")),
        ResetColor,
    )?;

    for i in off..(off + visible).min(items.len()) {
        if i == sel {
            queue!(
                out,
                SetForegroundColor(Color::Cyan),
                SetAttribute(Attribute::Bold),
                Print(format!("> {}\n", items[i])),
                SetAttribute(Attribute::Reset),
                ResetColor,
            )?;
        } else {
            queue!(out, Print(format!("  {}\n", items[i])))?;
        }
    }

    out.flush()?;
    Ok(())
}

fn erase(out: &mut impl Write, lines: usize) -> Result<(), Error> {
    if lines == 0 {
        return Ok(());
    }
    queue!(out, cursor::MoveUp(lines as u16))?;
    for _ in 0..lines {
        queue!(
            out,
            terminal::Clear(ClearType::CurrentLine),
            cursor::MoveDown(1),
        )?;
    }
    queue!(out, cursor::MoveUp(lines as u16))?;
    out.flush()?;
    Ok(())
}

fn parse_key(raw: &str) -> Option<(KeyCode, KeyModifiers)> {
    let (mods, rest) = parse_modifiers(raw.trim());
    match rest {
        "up" | "↑" => Some((KeyCode::Up, mods)),
        "down" | "↓" => Some((KeyCode::Down, mods)),
        "left" | "←" => Some((KeyCode::Left, mods)),
        "right" | "→" => Some((KeyCode::Right, mods)),
        "enter" | "return" => Some((KeyCode::Enter, mods)),
        "esc" | "escape" => Some((KeyCode::Esc, mods)),
        "space" => Some((KeyCode::Char(' '), mods)),
        "tab" => Some((KeyCode::Tab, mods)),
        "backspace" | "bs" => Some((KeyCode::Backspace, mods)),
        "delete" => Some((KeyCode::Delete, mods)),
        "home" => Some((KeyCode::Home, mods)),
        "end" => Some((KeyCode::End, mods)),
        "pageup" | "pgup" => Some((KeyCode::PageUp, mods)),
        "pagedown" | "pgdn" => Some((KeyCode::PageDown, mods)),
        s if s.len() == 1 => Some((KeyCode::Char(s.chars().next().unwrap()), mods)),
        _ => None,
    }
}

fn parse_modifiers(raw: &str) -> (KeyModifiers, &str) {
    let mut mods = KeyModifiers::NONE;
    let mut rest = raw;
    loop {
        let lower = rest.to_lowercase();
        if lower.starts_with("ctrl-") || lower.starts_with("ctrl+") {
            mods |= KeyModifiers::CONTROL;
            rest = &rest[5..];
        } else if lower.starts_with("alt-") || lower.starts_with("alt+") {
            mods |= KeyModifiers::ALT;
            rest = &rest[4..];
        } else if lower.starts_with("shift-") || lower.starts_with("shift+") {
            mods |= KeyModifiers::SHIFT;
            rest = &rest[6..];
        } else if lower.starts_with("cmd-") || lower.starts_with("cmd+") {
            mods |= KeyModifiers::SUPER;
            rest = &rest[4..];
        } else {
            break;
        }
    }
    (mods, rest)
}
