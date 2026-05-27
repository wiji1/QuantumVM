pub mod symbol;
pub mod keyword;
pub mod type_def;
pub mod identifier;
pub mod literal;

use symbol::{CompoundSymbol, Symbol};
use keyword::Keyword;
use type_def::TypeDefinition;
use identifier::Identifier;
use literal::Literal;
use crate::lexer::symbol::{CompoundAssignment, BLOCK_COMMENT};

pub(crate) trait TokenTrait: Sized {
    fn try_parse(input: &str) -> Option<(Self, usize)>;
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Symbol(Symbol),
    CompoundSymbol(CompoundSymbol),
    CompoundAssignment(CompoundAssignment),
    TypeDef(TypeDefinition),
    Keyword(Keyword),
    Identifier(Identifier),
    Literal(Literal),
}

pub(crate) fn match_keyword(input: &str, keyword: &str) -> Option<usize> {
    if input.starts_with(keyword) {
        let next_char = input.chars().nth(keyword.len());
        if next_char.map_or(true, |c| !c.is_alphanumeric() && c != '_') {
            return Some(keyword.len());
        }
    }
    None
}

pub struct Lexer {
    payload: String,
    current_line: usize,
    pub tokens: Vec<Token>,
    in_block_comment: bool,
}

#[derive(Clone, Debug)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

#[derive(Clone)]
pub struct Token {
    pub kind: TokenType,
    pub span: Span
}

impl Lexer  {
    pub(crate) fn new(payload: String) -> Lexer {
        Lexer { payload, current_line: 0, tokens: vec![], in_block_comment: false }
    }

    pub fn start(&mut self) {
        self.payload.clone().split("\n").for_each(|line| {
            let mut owned_string = line.to_owned();
            owned_string.push('\n');

            self.parse_line(&owned_string);
            self.current_line += 1;
        });
    }

    //TODO: Clean up this code
    fn parse_line(&mut self, line: &str) {
        let mut pos = 0;

        while pos < line.len() {
            if self.in_block_comment {
                if line[pos..].starts_with("*/") {
                    self.in_block_comment = false;
                    pos += 2;
                } else { pos += 1; }
                continue;
            }

            if line[pos..].starts_with("/*") {
                self.in_block_comment = true;
                pos += 2;
                continue;
            }

            let mut char_advance = 1;

            let mut span = {
                Span { line: self.current_line, col: pos, len: 0 }
            };

            if let Some((token, advance)) = CompoundAssignment::try_parse(&line[pos..]) {
                span.len = advance;

                self.tokens.push(Token { kind: TokenType::CompoundAssignment(token), span });
                char_advance = advance;
            } else if let Some((token, advance)) = CompoundSymbol::try_parse(&line[pos..]) {
                span.len = advance;

                self.tokens.push(Token { kind: TokenType::CompoundSymbol(token), span });
                char_advance = advance;
            } else if let Some((token, advance)) = TypeDefinition::try_parse(&line[pos..]) {
                span.len = advance;

                self.tokens.push(Token { kind: TokenType::TypeDef(token), span });
                char_advance = advance;
            } else if let Some((token, advance)) = Keyword::try_parse(&line[pos..]) {
                span.len = advance;

                self.tokens.push(Token { kind: TokenType::Keyword(token), span });
                char_advance = advance;
            } else if let Some((token, advance)) = TypeDefinition::try_parse(&line[pos..]) {
                span.len = advance;

                self.tokens.push(Token { kind: TokenType::TypeDef(token), span });
                char_advance = advance;
            } else if let Some((token, advance)) = Symbol::try_parse(&line[pos..]) {
                span.len = advance;

                self.tokens.push(Token { kind: TokenType::Symbol(token), span });
                char_advance = advance;
            } else if let Some((token, advance)) = Literal::try_parse(&line[pos..]) {
                span.len = advance;

                self.tokens.push(Token { kind: TokenType::Literal(token), span });
                char_advance = advance;
            } else if let Some((token, advance)) = Identifier::try_parse(&line[pos..]) {
                span.len = advance;

                self.tokens.push(Token { kind: TokenType::Identifier(token), span });
                char_advance = advance;
            }

            pos += char_advance;
        }
    }
}