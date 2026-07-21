use crossterm::style::{Attribute, Color};

use crate::buffer::Language;

// Как должен выглядить кусочек текста
pub struct Style {
    pub color: Option<Color>,
    pub attributes: Vec<Attribute>,
}

// Тип токена (ключевого слова)
pub enum TokenKind {
    Keyword,
    Comment,
    String,
    Number,
    Function,
    Plain,
}

// Одно правило подсветки
pub struct Rule {
    pub kind: TokenKind,
    pub definitions: Vec<String>, // Как именно распознает
    pub style: Style,
}

// Полное описание синтаксиса одного языка
pub struct SyntaxDefinition {
    language: Language,
    extensions: Vec<String>,
    rules: Vec<Rule>,
}
