use std::sync::LazyLock;
use regex::Regex;
use crate::enums::TokenTrait;

static IDENTIFIER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*").unwrap()
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