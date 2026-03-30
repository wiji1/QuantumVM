use crate::enums::identifier::Identifier;
use crate::enums::keyword::Keyword;
use crate::enums::literal::Literal;
use crate::enums::symbol::{CompoundSymbol, Symbol};
use crate::enums::type_def::TypeDefinition;

pub mod keyword;
pub mod type_def;
pub mod symbol;
pub mod identifier;
pub mod literal;

pub mod parse_error;
pub mod statement;
pub mod expression;
pub mod supporting_types;

pub(crate) trait TokenTrait: Sized {
    fn try_parse(input: &str) -> Option<(Self, usize)>;
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Symbol(Symbol),
    CompoundSymbol(CompoundSymbol),
    TypeDef(TypeDefinition),
    Keyword(Keyword),
    Identifier(Identifier),
    Literal(Literal)
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
