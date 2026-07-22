use std::{ffi::OsStr, fs, io, path::Path};

use crossterm::{
    event::{
        Event::{self, Key},
        KeyCode, KeyModifiers, read,
    },
    style::{Attribute, Color},
};

use crate::{
    buffer::Buffer,
    cursor::{Position, Viewport},
    syntax::{SyntaxDefinition, highlight_line, load_syntax, parse_attribute, parse_color},
    terminal::Terminal,
};

pub struct Editor {
    buffer: Buffer,
    cursor: Position,
    viewport: Viewport,
    should_quit: bool,
    status_message: String,
    syntaxes: Vec<SyntaxDefinition>,
    current_syntax: Option<usize>,
}

impl Editor {
    pub fn new(buffer: Buffer) -> Self {
        let mut syntaxes: Vec<SyntaxDefinition> = Vec::new();

        // Проходимся по JSON файлам синтаксиса и на основе их содержимого создаем SyntaxDefinition и добавляем в syntaxes
        if let Ok(entries) = fs::read_dir("syntaxes") {
            let valid_paths: Vec<String> = entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| entry.path().to_str().map(|path| path.to_string()))
                .collect();

            for path in valid_paths {
                if let Ok(def) = load_syntax(&path) {
                    syntaxes.push(def);
                }
            }
        }

        let mut editor = Self {
            should_quit: false,
            buffer,
            cursor: Position::new(),
            viewport: Viewport::new(),
            status_message: String::new(),
            syntaxes,
            current_syntax: None,
        };

        editor.select_syntax();
        editor
    }

    fn select_syntax(&mut self) {
        if let Some(filename) = &self.buffer.filename {
            let ext = Path::new(filename).extension().and_then(OsStr::to_str);
            for (i, syntax) in self.syntaxes.iter().enumerate() {
                // Если расширение текущего файла есть в поле extensions какого-то синтаксиса, то сохраняем индекс этого синтаксиса
                if let Some(ext) = ext
                    && syntax.extensions.contains(&ext.to_string())
                {
                    self.current_syntax = Some(i);
                    break;
                }
            }
        } else {
            self.current_syntax = None;
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
            self.status_message.clear();

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
                            None => self.set_message("Not found."),
                        }
                    } else {
                        self.set_message("Search cancelled.");
                        return Ok(());
                    }
                }
                KeyCode::Char('g') if event.modifiers == KeyModifiers::CONTROL => {
                    if let Some(input) = self.prompt("Go to line: ") {
                        if let Ok(line_num) = input.parse::<usize>() {
                            let target_line =
                                line_num.saturating_sub(1).min(self.buffer.line_count() - 1);

                            self.cursor.y = target_line;

                            let new_line_len = self.buffer.line_len(self.cursor.y);
                            if self.cursor.x > new_line_len {
                                self.cursor.x = new_line_len;
                            }

                            self.set_message(&format!("Jumping to line: {}", line_num));
                        } else {
                            self.set_message("Invalid line format.");
                            return Ok(());
                        }
                    } else {
                        return Ok(());
                    }
                }
                KeyCode::Char(ch) => {
                    self.buffer.insert_char(&self.cursor, ch);
                    self.cursor.x += 1;
                    self.buffer.modified = true;
                }
                KeyCode::Enter => {
                    let indent = self.buffer.get_line_indent(self.cursor.y);

                    self.buffer.insert_newline(&self.cursor);
                    // Переводим курсор на начало новой строки
                    self.cursor.x = 0;
                    self.cursor.y += 1;

                    // auto-indent (авто отступы)
                    for ch in indent.chars() {
                        self.buffer.insert_char(&self.cursor, ch);
                        self.cursor.x += 1;
                    }

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
                KeyCode::Left if event.modifiers == KeyModifiers::CONTROL => {
                    self.cursor.x = self.buffer.prev_word_boundary(self.cursor.x, self.cursor.y);
                }
                KeyCode::Left => {
                    if self.cursor.x > 0 {
                        self.cursor.x -= 1;
                    }
                }
                KeyCode::Right if event.modifiers == KeyModifiers::CONTROL => {
                    self.cursor.x = self.buffer.next_word_boundary(self.cursor.x, self.cursor.y);
                }
                KeyCode::Right => {
                    if self.cursor.x < self.buffer.line_len(self.cursor.y) {
                        self.cursor.x += 1;
                    }
                }
                _ => (),
            }
        }

        // Реализация "движения" "камеры" (viewport) за курсором.

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

        // Учитываем размер номера строк в файле
        let gutter_width = self.buffer.line_number_width() + 1;

        if self.cursor.x >= self.viewport.col_offset + width.saturating_sub(gutter_width) {
            self.viewport.col_offset = self.cursor.x - width.saturating_sub(gutter_width) + 1;
        }

        Ok(())
    }

    fn draw(&self) -> io::Result<()> {
        Terminal::clear_screen()?;
        Terminal::move_cursor_to(0, 0)?;

        // Учитываем размер номера строк в файле
        let gutter_width = self.buffer.line_number_width() + 1;

        let (width, height) = Terminal::size()?;
        let width = width as usize;
        let height = height as usize;

        // start_row и end_row - "границы" того, что мв видим сейчас на экране (с учетом размера терминала)
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

            Terminal::print_colored(&line_number_str, Color::Green)?;

            if let Some(idx) = self.current_syntax {
                for (text, style) in highlight_line(&visible_text, &self.syntaxes[idx]) {
                    let color = style.color.as_ref().and_then(|name| parse_color(name));

                    let mut attributes: Vec<Attribute> = Vec::new();
                    for name in &style.attributes {
                        if let Some(attr) = parse_attribute(name) {
                            attributes.push(attr);
                        }
                    }

                    Terminal::print_styled(&text, color, &attributes)?;
                }
            } else {
                Terminal::print(&visible_text)?;
            }
        }

        self.draw_status_bar()?;

        // Экранная координата = координата в тексте - сдвиг камеры
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

        // Заполняем пространство между двумя частями status bar'а пробелами
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
