use std::io::{self, stdout};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
};

pub struct Terminal {}

impl Terminal {
    pub fn terminate() -> io::Result<()> {
        disable_raw_mode()?;
        Ok(())
    }

    pub fn initialize() -> io::Result<()> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(0, 0)?;
        Ok(())
    }

    pub fn clear_screen() -> io::Result<()> {
        execute!(stdout(), Clear(ClearType::All))?;
        Ok(())
    }

    pub fn move_cursor_to(x: usize, y: usize) -> io::Result<()> {
        execute!(stdout(), MoveTo(x as u16, y as u16))?;
        Ok(())
    }

    pub fn print(line: &String) -> io::Result<()> {
        execute!(stdout(), Print(line))?;
        Ok(())
    }

    pub fn size() -> io::Result<(u16, u16)> {
        size()
    }
}
