use std::io::{self, stdout};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{
        Attribute, Attributes, Color, Print, ResetColor, SetAttribute, SetAttributes,
        SetForegroundColor,
    },
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode, size,
    },
};

pub struct Terminal {}

impl Terminal {
    pub fn terminate() -> io::Result<()> {
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn initialize() -> io::Result<()> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(0, 0)?;
        execute!(stdout(), EnterAlternateScreen)?;
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

    pub fn clear_current_line() -> io::Result<()> {
        execute!(stdout(), Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn print_colored(text: &str, color: Color) -> io::Result<()> {
        execute!(stdout(), SetForegroundColor(color), Print(text), ResetColor)?;
        Ok(())
    }

    pub fn print_styled(
        text: &str,
        foreground: Option<Color>,
        attributes: &[Attribute],
    ) -> io::Result<()> {
        let mut attrs = Attributes::default();
        for &attr in attributes {
            attrs.extend(Attributes::from(attr));
        }

        execute!(stdout(), SetAttributes(attrs))?;

        if let Some(color) = foreground {
            execute!(stdout(), SetForegroundColor(color))?;
        }

        execute!(
            stdout(),
            Print(text),
            SetAttribute(Attribute::Reset),
            ResetColor
        )?;

        Ok(())
    }
}
