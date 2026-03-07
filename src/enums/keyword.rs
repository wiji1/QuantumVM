use crate::enums::{match_keyword, TokenTrait};

#[derive(Debug)]
pub enum Keyword {
    OpenQASM,
    Measure
}

impl TokenTrait for Keyword {
    fn try_parse(input: &str) -> Option<(Self, usize)> {
        if let Some(len) = match_keyword(input, "OPENQASM") {
            return Some((Self::OpenQASM, len));
        }
        if let Some(len) = match_keyword(input, "measure") {
            return Some((Self::Measure, len));
        }
        None
    }
}
