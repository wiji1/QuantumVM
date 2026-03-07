use std::sync::LazyLock;
use regex::Regex;
use crate::enums::TokenTrait;

#[derive(Debug)]
pub enum Symbol {
    LBracket,
    RBracket,
    Semicolon,
    Dot,
    Comma,
    Equals,
    Slash,
    DecimalIntegerLiteral(String)
}

static DECIMAL_INTEGER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]+").unwrap());

impl TokenTrait for Symbol {
    fn try_parse(input: &str) -> Option<(Self, usize)> {
        if let Some(m) = DECIMAL_INTEGER.find(input) {
            return Some((Symbol::DecimalIntegerLiteral(m.as_str().to_string()), m.len()));
        }

        let c = input.chars().next()?;

        match c {
            '[' => Some((Symbol::LBracket, 1)),
            ']' => Some((Symbol::RBracket, 1)),
            ';' => Some((Symbol::Semicolon, 1)),
            '.' => Some((Symbol::Dot, 1)),
            ',' => Some((Symbol::Comma, 1)),
            '=' => Some((Symbol::Equals, 1)),
            '/' => Some((Symbol::Slash, 1)),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum CompoundSymbol {
    LineComment
}

static LINE_COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^//[^\r\n]*").unwrap());

impl TokenTrait for CompoundSymbol {
    fn try_parse(input: &str) -> Option<(Self, usize)> {
        if let Some(m) = LINE_COMMENT.find(input) {
            return Some((CompoundSymbol::LineComment, m.len()));
        }

        None
    }
}