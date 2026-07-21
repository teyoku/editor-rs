use std::fs;

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
    pub language: Language,
    pub extensions: Vec<String>,
    pub rules: Vec<Rule>,
}

pub fn load_syntax(path: &str) -> Result<SyntaxDefinition, String> {
    let content = fs::read_to_string(path).map_err(|err| format!("File read error: {err}"))?;
    let definition =
        serde_json::from_str(&content).map_err(|err| format!("File parsing error: {err}"))?;

    Ok(definition)
}
