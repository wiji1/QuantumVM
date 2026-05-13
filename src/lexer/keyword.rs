use crate::lexer::{match_keyword, TokenTrait};

#[derive(Debug, Clone)]
pub enum Keyword {
    OpenQASM,
    Measure,
    Gate,
    If,
    Else,
    Reset,
    Barrier,
    For,
    While,
    In,
    Const,
    Continue,
    Break,
    Return,
}

impl TokenTrait for Keyword {
    fn try_parse(input: &str) -> Option<(Self, usize)> {
        if let Some(len) = match_keyword(input, "OPENQASM") {
            return Some((Self::OpenQASM, len));
        }
        if let Some(len) = match_keyword(input, "measure") {
            return Some((Self::Measure, len));
        }
        if let Some(len) = match_keyword(input, "gate") {
            return Some((Self::Gate, len));
        }
        if let Some(len) = match_keyword(input, "if") {
            return Some((Self::If, len));
        }
        if let Some(len) = match_keyword(input, "else") {
            return Some((Self::Else, len));
        }
        if let Some(len) = match_keyword(input, "reset") {
            return Some((Self::Reset, len));
        }
        if let Some(len) = match_keyword(input, "barrier") {
            return Some((Self::Barrier, len));
        }
        if let Some(len) = match_keyword(input, "for") {
            return Some((Self::For, len));
        }
        if let Some(len) = match_keyword(input, "while") {
            return Some((Self::While, len));
        }
        if let Some(len) = match_keyword(input, "in") {
            return Some((Self::In, len));
        }
        if let Some(len) = match_keyword(input, "const") {
            return Some((Self::Const, len));
        }
        if let Some(len) = match_keyword(input, "continue") {
            return Some((Self::Continue, len));
        }
        if let Some(len) = match_keyword(input, "break") {
            return Some((Self::Break, len));
        }
        if let Some(len) = match_keyword(input, "return") {
            return Some((Self::Return, len));
        }
        None
    }
}
