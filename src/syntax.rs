use crossterm::style::{Attribute, Color};
use serde::Deserialize;

use crate::buffer::Language;

// Как должен выглядить кусочек текста
#[derive(Deserialize)]
pub struct Style {
    pub color: Option<String>,
    pub attributes: Vec<String>,
}

// Тип токена (ключевого слова)
#[derive(Deserialize)]
pub enum TokenKind {
    Keyword,
    Comment,
    String,
    Number,
    Function,
    Plain,
}

// Одно правило подсветки
#[derive(Deserialize)]
pub struct Rule {
    pub kind: TokenKind,
    pub definitions: Vec<String>, // Как именно распознает
    pub style: Style,
}

// Полное описание синтаксиса одного языка
#[derive(Deserialize)]
pub struct SyntaxDefinition {
    language: Language,
    extensions: Vec<String>,
    rules: Vec<Rule>,
}
