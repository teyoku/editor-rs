use std::{fs, io};

use crate::cursor::Position;

pub struct Buffer {
    pub lines: Vec<String>,
    pub filename: Option<String>,
    pub modified: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            filename: None,
            modified: false,
        }
    }

    pub fn from_file(path: &str) -> Self {
        let mut lines = match fs::read_to_string(path) {
            Ok(content) => content.lines().map(|line| line.to_string()).collect(),
            Err(_) => vec![String::new()],
        };

        if lines.len() == 0 {
            lines.push(String::new());
        }

        Self {
            lines,
            filename: Some(path.to_string()),
            modified: false,
        }
    }

    pub fn save(&self) -> io::Result<()> {
        match &self.filename {
            Some(path) => fs::write(path, self.lines.join("\n")),
            None => Ok(()),
        }
    }

    pub fn insert_char(&mut self, pos: &Position, ch: char) {
        if pos.x == self.line_len(pos.y) {
            self.lines[pos.y].push(ch);
        } else {
            if let Some((idx, _)) = self.lines[pos.y].char_indices().nth(pos.x) {
                self.lines[pos.y].insert(idx, ch);
            }
        }
    }

    pub fn insert_newline(&mut self, pos: &Position) {
        let current_line = &self.lines[pos.y];

        let byte_idx = current_line
            .char_indices()
            .map(|(b_idx, _)| b_idx)
            .nth(pos.x)
            .unwrap_or(current_line.len());

        let left = current_line[..byte_idx].to_string();
        let right = current_line[byte_idx..].to_string();

        // Оставляем левую часть
        self.lines[pos.y] = left;

        // Ставим правую часть на позицию ниже
        self.lines.insert(pos.y + 1, right);
    }

    pub fn delete_char(&mut self, pos: &mut Position) {
        if pos.x > 0 {
            let current_line = &mut self.lines[pos.y];

            //  Находим байтовый индекс символа, который стоит ПЕРЕД курсором
            if let Some((byte_idx, _)) = current_line.char_indices().nth(pos.x - 1) {
                current_line.remove(byte_idx);
                // Сдвигаем курсор влево на один символ
                pos.x -= 1;
            }
        } else if pos.y > 0 {
            // Удаляем текущую строку из вектора и забираем её содержимое
            let current_line_text = self.lines.remove(pos.y);

            // Переходим на строку выше
            pos.y -= 1;

            // Запоминаем, сколько символов было в предыдущей строке
            let prev_line_len_chars = self.line_len(pos.y);

            // Приклеиваем текст удаленной строки к концу предыдущей строки
            self.lines[pos.y].push_str(&current_line_text);

            // Обновляем позицию курсора на место стыка
            pos.x = prev_line_len_chars;
        }
    }

    pub fn get_line_indent(&self, y: usize) -> String {
        self.lines[y]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect::<String>()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line_len(&self, y: usize) -> usize {
        self.lines[y].chars().count()
    }

    pub fn line_number_width(&self) -> usize {
        self.line_count().to_string().len()
    }
}
