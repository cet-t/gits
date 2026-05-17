use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::error::Error;

/// Show an interactive selector on stderr and return the index of the chosen item.
///
/// stdout is never written to, keeping it clean for value-generation commands.
pub fn select(items: &[String], prompt: &str) -> Result<usize, Error> {
    if items.is_empty() {
        return Err(Error::Empty);
    }

    let stderr = io::stderr();
    let mut out = stderr.lock();

    terminal::enable_raw_mode()?;
    let result = run_select(&mut out, items, prompt);

    // Always restore terminal state even on error.
    let _ = execute!(out, cursor::Show);
    terminal::disable_raw_mode()?;

    result
}

fn run_select(out: &mut impl Write, items: &[String], prompt: &str) -> Result<usize, Error> {
    let (_, rows) = terminal::size()?;
    let mut visible = (rows as usize).saturating_sub(2).max(1).min(items.len());

    let mut sel = 0usize;
    let mut off = 0usize;

    // Drain buffered key events (e.g. the Enter that launched this process on Windows)
    while event::poll(std::time::Duration::ZERO)? {
        event::read()?;
    }

    execute!(out, cursor::Hide)?;
    render(out, items, sel, off, visible, prompt)?;

    loop {
        match event::read()? {
            Event::Key(key) => match (key.code, key.modifiers) {
                (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                    if sel > 0 {
                        sel -= 1;
                        if sel < off {
                            off = sel;
                        }
                        erase(out, visible + 1)?;
                        render(out, items, sel, off, visible, prompt)?;
                    }
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                    if sel + 1 < items.len() {
                        sel += 1;
                        if sel >= off + visible {
                            off = sel + 1 - visible;
                        }
                        erase(out, visible + 1)?;
                        render(out, items, sel, off, visible, prompt)?;
                    }
                }
                (KeyCode::Enter, _) | (KeyCode::Char('l'), KeyModifiers::NONE) => {
                    erase(out, visible + 1)?;
                    return Ok(sel);
                }
                (KeyCode::Esc, _)
                | (KeyCode::Char('h'), KeyModifiers::NONE)
                | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    erase(out, visible + 1)?;
                    return Err(Error::Cancelled);
                }
                _ => {}
            },
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
        Print(format!("  {prompt}\r\n")),
        ResetColor,
    )?;

    for i in off..(off + visible).min(items.len()) {
        if i == sel {
            queue!(
                out,
                SetForegroundColor(Color::Cyan),
                SetAttribute(Attribute::Bold),
                Print(format!("> {}\r\n", items[i])),
                SetAttribute(Attribute::Reset),
                ResetColor,
            )?;
        } else {
            queue!(out, Print(format!("  {}\r\n", items[i])))?;
        }
    }

    out.flush()?;
    Ok(())
}

/// Erase `lines` lines upward from the current cursor position, then return
/// the cursor to where those lines began.
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
