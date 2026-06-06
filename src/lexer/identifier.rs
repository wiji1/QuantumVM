use std::sync::LazyLock;
use regex::Regex;
use crate::lexer::TokenTrait;

static IDENTIFIER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[\p{L}_][\p{L}\p{N}_]*").unwrap()
});


#[derive(Debug, Clone)]
pub enum Identifier {
    Identifier(String),
}

impl TokenTrait for Identifier {
    fn try_parse(input: &str) -> Option<(Self, usize)> {
        let m = IDENTIFIER.find(input)?;
        Some((Identifier::Identifier(m.as_str().to_string()), m.len()))
    }
}