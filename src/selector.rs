use std::io::Write;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::ansi;
use crate::error::GitsError;
use crate::{config, error::GitsResult};

#[derive(Debug, Clone, Default)]
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

pub fn select_with_keymap(
    items: &[String],
    prompt: &str,
    key_codes: &KeyCodes,
) -> GitsResult<usize> {
    if items.is_empty() {
        return Err(GitsError::Empty);
    }

    // Stderr is unbuffered; buffer so each frame reaches the terminal in one write.
    let mut out = std::io::BufWriter::new(std::io::stderr().lock());

    terminal::enable_raw_mode()?;
    let result = run_select(&mut out, items, prompt, key_codes);

    let _ = execute!(out, terminal::EnableLineWrap, cursor::Show);
    terminal::disable_raw_mode()?;

    result
}

fn run_select(
    out: &mut impl Write,
    items: &[String],
    prompt: &str,
    key_codes: &KeyCodes,
) -> GitsResult<usize> {
    let (_, rows) = terminal::size()?;
    let mut visible = (rows as usize).saturating_sub(2).max(1).min(items.len());

    let mut sel = 0usize;
    let mut off = 0usize;

    // Discard input left over from launching the command, before the list is shown.
    drain_events();

    // Long items are cut at the right edge instead of wrapping, so each item
    // occupies exactly one line and the redraw line count stays correct.
    execute!(out, cursor::Hide, terminal::DisableLineWrap)?;
    render(out, items, sel, off, visible, prompt, 0)?;

    loop {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let matched = (key.code, key.modifiers);

                let is_up = bindings_contain(&key_codes.up_keys, matched)
                    || (key_codes.up_keys.is_empty()
                        && matches!(
                            matched,
                            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE)
                        ));

                let is_down = bindings_contain(&key_codes.down_keys, matched)
                    || (key_codes.down_keys.is_empty()
                        && matches!(
                            matched,
                            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE)
                        ));

                let is_confirm = bindings_contain(&key_codes.confirm_keys, matched)
                    || (key_codes.confirm_keys.is_empty()
                        && matches!(
                            matched,
                            (KeyCode::Enter, _) | (KeyCode::Char('l'), KeyModifiers::NONE)
                        ));

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
                        render(out, items, sel, off, visible, prompt, visible + 1)?;
                    }
                } else if is_down {
                    if sel + 1 < items.len() {
                        sel += 1;
                        if sel >= off + visible {
                            off = sel + 1 - visible;
                        }
                        render(out, items, sel, off, visible, prompt, visible + 1)?;
                    }
                } else if is_confirm {
                    erase(out, visible + 1)?;
                    return Ok(sel);
                } else if is_cancel {
                    erase(out, visible + 1)?;
                    return Err(GitsError::Cancelled);
                }
            }
            Event::Resize(_, new_rows) => {
                let new_visible = (new_rows as usize)
                    .saturating_sub(2)
                    .max(1)
                    .min(items.len());
                let prev_lines = visible + 1;
                visible = new_visible;
                if sel >= off + visible {
                    off = sel + 1 - visible;
                }
                render(out, items, sel, off, visible, prompt, prev_lines)?;
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
    bindings.contains(&target)
}

/// Draws the list over the previous frame (`prev_lines` lines above the cursor)
/// in a single flush. Lines are overwritten in place rather than cleared first,
/// so no blank frame is ever shown.
fn render(
    out: &mut impl Write,
    items: &[String],
    sel: usize,
    off: usize,
    visible: usize,
    prompt: &str,
    prev_lines: usize,
) -> GitsResult<()> {
    queue!(out, terminal::BeginSynchronizedUpdate)?;
    if prev_lines > 0 {
        queue!(out, cursor::MoveUp(prev_lines as u16))?;
    }
    queue!(
        out,
        cursor::MoveToColumn(0),
        SetForegroundColor(Color::DarkYellow),
        Print(format!("  {prompt}")),
        ResetColor,
        terminal::Clear(ClearType::UntilNewLine),
        Print("\r\n"),
    )?;

    #[allow(clippy::needless_range_loop)]
    for i in off..(off + visible).min(items.len()) {
        if i == sel {
            // Drop the item's own colors so the whole row shows the highlight.
            queue!(
                out,
                SetForegroundColor(Color::Cyan),
                SetAttribute(Attribute::Bold),
                Print(format!("> {}", ansi::strip(&items[i]))),
                SetAttribute(Attribute::Reset),
                ResetColor,
            )?;
        } else {
            queue!(
                out,
                Print(format!("  {}", items[i])),
                SetAttribute(Attribute::Reset),
                ResetColor,
            )?;
        }
        queue!(out, terminal::Clear(ClearType::UntilNewLine), Print("\r\n"))?;
    }

    // Remove leftover lines when the frame shrank (e.g. after a resize).
    queue!(
        out,
        terminal::Clear(ClearType::FromCursorDown),
        terminal::EndSynchronizedUpdate,
    )?;
    out.flush()?;
    Ok(())
}

fn erase(out: &mut impl Write, lines: usize) -> GitsResult<()> {
    if lines == 0 {
        return Ok(());
    }
    queue!(
        out,
        cursor::MoveUp(lines as u16),
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::FromCursorDown),
    )?;
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
