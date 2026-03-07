use crate::enums::identifier::Identifier;
use crate::enums::keyword::Keyword;
use crate::enums::symbol::{CompoundSymbol, Symbol};
use crate::enums::type_def::TypeDefinition;

pub mod keyword;
pub mod type_def;
pub mod symbol;
pub(crate) mod identifier;

pub(crate) trait TokenTrait: Sized {
    fn try_parse(input: &str) -> Option<(Self, usize)>;
}

#[derive(Debug)]
pub enum TokenType {
    Symbol(Symbol),
    CompoundSymbol(CompoundSymbol),
    TypeDef(TypeDefinition),
    Keyword(Keyword),
    Identifier(Identifier),
}

fn match_keyword(input: &str, keyword: &str) -> Option<usize> {
    if input.starts_with(keyword) {
        let next_char = input.chars().nth(keyword.len());
        if next_char.map_or(true, |c| !c.is_alphanumeric() && c != '_') {
            return Some(keyword.len());
        }
    }
    None
}
