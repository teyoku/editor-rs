use std::fs;

use crossterm::style::{Attribute, Color};
use serde::Deserialize;

use unicode_segmentation::UnicodeSegmentation;

use crate::buffer::Language;

// Как должен выглядить кусочек текста
#[derive(Deserialize, Clone)]
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

pub fn highlight_line(line: &str, syntax: &SyntaxDefinition) -> Vec<(String, Style)> {
    let mut segments: Vec<(String, Style)> = Vec::new();

    for part in line.split_word_bounds() {
        match syntax
            .rules
            .iter()
            .find(|&rule| rule.definitions.contains(&part.to_string()))
        {
            Some(rule) => segments.push((part.to_string(), rule.style.clone())),
            None => segments.push((
                part.to_string(),
                Style {
                    color: None,
                    attributes: Vec::new(),
                },
            )),
        }
    }
    segments
}

pub fn parse_color(name: &str) -> Option<Color> {
    match name {
        "Black" => Some(Color::Black),
        "DarkGrey" => Some(Color::DarkGrey),
        "Red" => Some(Color::Red),
        "DarkRed" => Some(Color::DarkRed),
        "Green" => Some(Color::Green),
        "DarkGreen" => Some(Color::DarkGreen),
        "Yellow" => Some(Color::Yellow),
        "DarkYellow" => Some(Color::DarkYellow),
        "Blue" => Some(Color::Blue),
        "DarkBlue" => Some(Color::DarkBlue),
        "Magenta" => Some(Color::Magenta),
        "DarkMagenta" => Some(Color::DarkMagenta),
        "Cyan" => Some(Color::Cyan),
        "DarkCyan" => Some(Color::DarkCyan),
        "White" => Some(Color::White),
        "Grey" => Some(Color::Grey),
        _ => None,
    }
}

pub fn parse_attribute(name: &str) -> Option<Attribute> {
    match name {
        "Bold" => Some(Attribute::Bold),
        "Dim" => Some(Attribute::Dim),
        "Italic" => Some(Attribute::Italic),
        "Underlined" => Some(Attribute::Underlined),
        "DoubleUnderlined" => Some(Attribute::DoubleUnderlined),
        "Undercurled" => Some(Attribute::Undercurled),
        "Underdotted" => Some(Attribute::Underdotted),
        "Underdashed" => Some(Attribute::Underdashed),
        "SlowBlink" => Some(Attribute::SlowBlink),
        "RapidBlink" => Some(Attribute::RapidBlink),
        "Reverse" => Some(Attribute::Reverse),
        "Fraktur" => Some(Attribute::Fraktur),
        "Framed" => Some(Attribute::Framed),
        "Encircled" => Some(Attribute::Encircled),
        "OverLined" => Some(Attribute::OverLined),
        "CrossedOut" => Some(Attribute::CrossedOut),
        _ => None,
    }
}
