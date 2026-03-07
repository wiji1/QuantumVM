use crate::enums::symbol::{CompoundSymbol, Symbol};
use crate::enums::{TokenTrait, TokenType};
use crate::enums::keyword::Keyword;
use crate::enums::type_def::TypeDefinition;
use crate::enums::identifier::Identifier;

pub struct Lexer {
    payload: String,
    current_line: usize,
    tokens: Vec<Token>,
}

pub struct Span {
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

pub struct Token {
    pub kind: TokenType,
    pub span: Span
}

impl Lexer {
    pub(crate) fn new(payload: String) -> Lexer {
        Lexer { payload, current_line: 0, tokens: vec![] }
    }

    pub fn start(&mut self) {
        self.payload.clone().split("\n").for_each(|line| {
            self.parse_line(line);
            self.current_line += 1;
        });

        println!("Finished Lexing!");

        for x in &self.tokens {
            println!("{:?}", x.kind);
        }
    }

    fn parse_line(&mut self, line: &str) {
        let mut pos = 0;

        while pos < line.len() {
            let mut char_advance = 1;

            let mut span = {
                Span { line: self.current_line, col: pos, len: 0 }
            };

            if let Some((token, advance)) = CompoundSymbol::try_parse(&line[pos..]) {
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
            } else if let Some((token, advance)) = Identifier::try_parse(&line[pos..]) {
                span.len = advance;

                self.tokens.push(Token { kind: TokenType::Identifier(token), span });
                char_advance = advance;
            }

            pos += char_advance;
        }
    }



}