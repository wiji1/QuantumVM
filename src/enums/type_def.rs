use crate::enums::{match_keyword, TokenTrait};

#[derive(Debug)]
pub enum TypeDefinition {
    Qubit,
    Bit
}

impl TokenTrait for TypeDefinition {
    fn try_parse(input: &str) -> Option<(Self, usize)> {
        if let Some(len) = match_keyword(input, "qubit") {
            return Some((Self::Qubit, len));
        }
        if let Some(len) = match_keyword(input, "bit") {
            return Some((Self::Bit, len));
        }
        None
    }
}