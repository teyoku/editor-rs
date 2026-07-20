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
    status_message: String,
}

impl Editor {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            should_quit: false,
            buffer,
            cursor: Position::new(),
            viewport: Viewport::new(),
            status_message: String::new(),
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
        Terminal::move_cursor_to(0, 0)?;
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
                    if self.buffer.filename.is_none() {
                        if let Some(path) = self.prompt("Save as: ") {
                            self.buffer.filename = Some(path);
                        } else {
                            self.set_message("Save aborted.");
                            return Ok(());
                        }
                    }

                    self.buffer.save()?;
                    self.buffer.modified = false;
                    self.set_message("File saved successfully.");
                }
                KeyCode::Char('f') if event.modifiers == KeyModifiers::CONTROL => {
                    if let Some(query) = self.prompt("Search: ") {
                        match self.find_text(&query) {
                            Some(pos) => self.cursor = pos,
                            None => self.set_message("Not found"),
                        }
                    } else {
                        self.set_message("Search cancelled.");
                        return Ok(());
                    }
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

        let gutter_width = self.buffer.line_number_width() + 1;

        if self.cursor.x >= self.viewport.col_offset + width.saturating_sub(gutter_width) {
            self.viewport.col_offset = self.cursor.x - width.saturating_sub(gutter_width) + 1;
        }

        Ok(())
    }

    fn draw(&self) -> io::Result<()> {
        Terminal::clear_screen()?;
        Terminal::move_cursor_to(0, 0)?;

        let gutter_width = self.buffer.line_number_width() + 1;

        let (width, height) = Terminal::size()?;
        let width = width as usize;
        let height = height as usize;

        let start_row = self.viewport.row_offset;
        let end_row = (start_row + height - 1).clamp(0, self.buffer.line_count());

        let visible_lines = &self.buffer.lines[start_row..end_row];
        for (screen_y, line) in visible_lines.iter().enumerate() {
            Terminal::move_cursor_to(0, screen_y)?;

            let visible_text = line
                .chars()
                .skip(self.viewport.col_offset)
                .take(width.saturating_sub(gutter_width))
                .collect::<String>();

            let number = screen_y + self.viewport.row_offset + 1;
            let line_number_str = format!(
                "{:>width$} ",
                number,
                width = self.buffer.line_number_width()
            );

            Terminal::print(&line_number_str)?;
            Terminal::print(&visible_text)?;
        }

        self.draw_status_bar()?;

        let screen_x = self.cursor.x - self.viewport.col_offset + gutter_width;
        let screen_y = self.cursor.y - self.viewport.row_offset;

        Terminal::move_cursor_to(screen_x, screen_y)?;

        Ok(())
    }

    fn draw_status_bar(&self) -> io::Result<()> {
        let (width, height) = Terminal::size()?;
        let width = width as usize;
        let height = height as usize;

        let mut left_part = self.status_message.clone();

        if left_part.is_empty() {
            left_part = match &self.buffer.filename {
                Some(name) => format!("[{name}]"),
                None => "[No Name]".to_string(),
            };
        }

        if self.buffer.modified {
            left_part.push_str(" [modified]");
        }

        let right_part = format!(
            "Ln {}/{} Col {}",
            self.cursor.y + 1,
            self.buffer.line_count(),
            self.cursor.x + 1,
        );

        let mut status_line = String::new();

        status_line.push_str(&left_part);

        while status_line.chars().count() < width - right_part.chars().count() {
            status_line.push(' ');
        }

        status_line.push_str(&right_part);

        Terminal::move_cursor_to(0, height - 1)?;

        Terminal::clear_current_line()?;
        Terminal::print(&status_line)?;

        Ok(())
    }

    fn set_message(&mut self, msg: &str) {
        self.status_message = msg.to_string();
    }

    fn prompt(&mut self, promt_text: &str) -> Option<String> {
        let (_, height) = Terminal::size().unwrap();
        let height = height as usize;

        let mut input = String::new();
        loop {
            Terminal::move_cursor_to(0, height - 1).unwrap();
            Terminal::clear_current_line().unwrap();
            Terminal::print(&format!("{}{}", promt_text, input)).unwrap();

            if let Ok(Key(event)) = read() {
                match event.code {
                    KeyCode::Char(ch) => input.push(ch),
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Enter => return Some(input),
                    KeyCode::Esc => return None,
                    _ => (),
                }
            }
        }
    }

    fn find_text(&self, query: &str) -> Option<Position> {
        for (y, line) in self.buffer.lines.iter().enumerate() {
            if let Some(byte_idx) = line.find(query) {
                // Ищем x - длину найденной подстроки (работаем с байтовым индексом)
                let x = &line[..byte_idx].chars().count();
                return Some(Position { x: *x, y });
            }
        }

        None
    }
}
