use std::io::{self, Write, stdout};

use crossterm::event::{
    Event::{self, Key},
    KeyCode, KeyModifiers, read,
};

use crate::{buffer::Buffer, cursor::Position, terminal::Terminal};

pub struct Editor {
    buffer: Buffer,
    cursor: Position,
    should_quit: bool,
}

impl Editor {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            should_quit: false,
            buffer,
            cursor: Position::new(),
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

        Ok(())
    }

    fn draw(&self) -> io::Result<()> {
        Terminal::clear_screen()?;
        Terminal::move_cursor_to(0, 0)?;

        for (i, line) in self.buffer.lines.iter().enumerate() {
            Terminal::move_cursor_to(0, i)?;
            Terminal::print(line)?;
        }

        Terminal::move_cursor_to(self.cursor.x, self.cursor.y)?;

        Ok(())
    }
}
