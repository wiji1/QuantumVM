use crate::lexer::{TokenType, Span};
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        expected: TokenType,
        found: TokenType,
        span: Span,
    },
    UnexpectedEof {
        expected: TokenType,
    },
    InvalidLiteral {
        message: String,
        span: Span,
    },
    InvalidVersion {
        found: String,
        span: Span,
    },
    InvalidStatement {
        found: TokenType,
        span: Span,
    },
    TypeError {
        message: String,
        span: Span,
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found, span } => {
                write!(
                    f,
                    "unexpected token at line {}, col {} — expected {:?}, found {:?}",
                    span.line, span.col, expected, found
                )
            }
            ParseError::UnexpectedEof { expected } => {
                write!(f, "unexpected end of file — expected {:?}", expected)
            }
            ParseError::InvalidLiteral { message, span } => {
                write!(
                    f,
                    "invalid literal at line {}, col {}: {}",
                    span.line, span.col, message
                )
            }
            ParseError::InvalidVersion { found, span } => {
                write!(
                    f,
                    "invalid version specifier '{}' at line {}, col {}",
                    found, span.line, span.col
                )
            },
            ParseError::InvalidStatement { found, span } => {
                write!(
                    f,
                    "invalid statement — found '{:?}' at line {}, col {}",
                    found, span.line, span.col
                )
            },
            ParseError::TypeError { message, span } => {
                write!(
                    f,
                    "type error: {:?} at line {}, col {}",
                    message, span.line, span.col
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}