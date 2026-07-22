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
#[derive(Deserialize, PartialEq, Eq)]
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

fn highlight_tokens(text: &str, syntax: &SyntaxDefinition) -> Vec<(String, Style)> {
    let mut segments: Vec<(String, Style)> = Vec::new();

    for part in text.split_word_bounds() {
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

pub fn highlight_line(line: &str, syntax: &SyntaxDefinition) -> Vec<(String, Style)> {
    let mut segments: Vec<(String, Style)> = Vec::new();

    // Все правила синтаксиса для комментариев
    let comment_rules: Vec<&Rule> = syntax
        .rules
        .iter()
        .filter(|rule| rule.kind == TokenKind::Comment)
        .collect();

    let mut found_idx: Option<usize> = None;
    let mut found_style: Option<&Style> = None;

    // Ищем байтовых индекс первого символа комментария в строке
    // Также в found_style сохраняем стиль для этого комментария
    for rule in comment_rules {
        for definition in &rule.definitions {
            if let Some(byte_idx) = line.find(definition) {
                found_idx = match found_idx {
                    Some(pos) => Some(pos.min(byte_idx)),
                    None => Some(byte_idx),
                };
                found_style = Some(&rule.style);
            }
        }
    }

    if let Some(byte_idx) = found_idx {
        // Текст ДО комментария - выделяет подсветкой для языка
        let unstyled_segment = &line[..byte_idx];
        segments.extend(highlight_tokens(unstyled_segment, syntax));

        // Сам символ комментария и текст ПОСЛЕ него - выделяем подсветкой комментария
        let styled_segment = &line[byte_idx..];
        segments.push((styled_segment.to_string(), found_style.unwrap().clone()));
    } else {
        segments.extend(highlight_tokens(line, syntax));
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
