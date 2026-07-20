use std::io;

use crossterm::event::{
    Event::{self, Key},
    KeyCode, KeyModifiers, read,
};

use crate::{
    buffer::Buffer,
    cursor::{Position, Viewport},
    terminal::Terminal,
};

pub struct Editor {
    buffer: Buffer,
    cursor: Position,
    viewport: Viewport,
    should_quit: bool,
}

impl Editor {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            should_quit: false,
            buffer,
            cursor: Position::new(),
            viewport: Viewport::new(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        Terminal::initialize()?;
        loop {
            if self.should_quit {
                break;
            }
            self.draw()?;

            let event = read()?;
            self.evaluate_event(&event)?;
        }
        Terminal::clear_screen()?;
        Terminal::terminate()?;
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> io::Result<()> {
        if let Key(event) = event {
            match event.code {
                KeyCode::Char('q') if event.modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                KeyCode::Char('s') if event.modifiers == KeyModifiers::CONTROL => {
                    self.buffer.save()?;
                    self.buffer.modified = false;
                }
                KeyCode::Char(ch) => {
                    self.buffer.insert_char(&self.cursor, ch);
                    self.cursor.x += 1;
                    self.buffer.modified = true;
                }
                KeyCode::Enter => {
                    self.buffer.insert_newline(&self.cursor);

                    // Переводим курсор на начало новой строки
                    self.cursor.x = 0;
                    self.cursor.y += 1;

                    self.buffer.modified = true;
                }
                KeyCode::Backspace => {
                    self.buffer.delete_char(&mut self.cursor);
                    self.buffer.modified = true;
                }
                KeyCode::Up => {
                    if self.cursor.y > 0 {
                        self.cursor.y -= 1;

                        let current_line_len = self.buffer.line_len(self.cursor.y);
                        if self.cursor.x > current_line_len {
                            self.cursor.x = current_line_len;
                        }
                    }
                }
                KeyCode::Down => {
                    if self.cursor.y + 1 < self.buffer.line_count() {
                        self.cursor.y += 1;

                        let current_line_len = self.buffer.line_len(self.cursor.y);
                        if self.cursor.x > current_line_len {
                            self.cursor.x = current_line_len;
                        }
                    }
                }
                KeyCode::Left => {
                    if self.cursor.x > 0 {
                        self.cursor.x -= 1;
                    }
                }
                KeyCode::Right => {
                    if self.cursor.x < self.buffer.line_len(self.cursor.y) {
                        self.cursor.x += 1;
                    }
                }
                _ => (),
            }
        }

        let (width, height) = Terminal::size()?;
        let width = width as usize;
        let height = height as usize;

        if self.cursor.y < self.viewport.row_offset {
            self.viewport.row_offset = self.cursor.y;
        }

        if self.cursor.y >= self.viewport.row_offset + height - 1 {
            self.viewport.row_offset = self.cursor.y - (height - 1) + 1;
        }

        if self.cursor.x < self.viewport.col_offset {
            self.viewport.col_offset = self.cursor.x;
        }

        if self.cursor.x >= self.viewport.col_offset + width {
            self.viewport.col_offset = self.cursor.x - width + 1;
        }

        Ok(())
    }

    fn draw(&self) -> io::Result<()> {
        Terminal::clear_screen()?;
        Terminal::move_cursor_to(0, 0)?;

        let (width, height) = Terminal::size()?;
        let width = width as usize;
        let height = height as usize;

        let start_row = self.viewport.row_offset;
        let end_row = (start_row + height).clamp(0, self.buffer.line_count());

        let visible_lines = &self.buffer.lines[start_row..end_row];
        for (screen_y, line) in visible_lines.iter().enumerate() {
            Terminal::move_cursor_to(0, screen_y)?;

            let visible_text = line
                .chars()
                .skip(self.viewport.col_offset)
                .take(width)
                .collect::<String>();
            Terminal::print(&visible_text)?;
        }

        let screen_x = self.cursor.x - self.viewport.col_offset;
        let screen_y = self.cursor.y - self.viewport.row_offset;

        Terminal::move_cursor_to(screen_x, screen_y)?;

        Ok(())
    }
}
