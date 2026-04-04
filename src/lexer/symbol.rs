use std::sync::LazyLock;
use regex::Regex;
use crate::lexer::TokenTrait;
use crate::parser::supporting_types::BinaryOp;

#[derive(Debug, Clone)]
pub enum Symbol {
    LBracket,
    RBracket,
    Colon,
    Semicolon,
    Dot,
    Comma,
    Equals,
    Slash,
    NewLine,
    Star,
    Plus,
    Minus,
    Tilde,
    Bang,
    Percent,
    LessThan,
    GreaterThan,
    Ampersand,
    Caret,
    Pipe,
    LParen,
    RParen,
    LBrace,
    RBrace,
}

impl TokenTrait for Symbol {
    fn try_parse(input: &str) -> Option<(Self, usize)> {
        let c = input.chars().next()?;

        match c {
            '[' => Some((Symbol::LBracket, 1)),
            ']' => Some((Symbol::RBracket, 1)),
            ';' => Some((Symbol::Semicolon, 1)),
            ':' => Some((Symbol::Colon, 1)),
            '.' => Some((Symbol::Dot, 1)),
            ',' => Some((Symbol::Comma, 1)),
            '=' => Some((Symbol::Equals, 1)),
            '/' => Some((Symbol::Slash, 1)),
            '\n' => Some((Symbol::NewLine, 1)),
            '*' => Some((Symbol::Star, 1)),
            '+' => Some((Symbol::Plus, 1)),
            '-' => Some((Symbol::Minus, 1)),
            '~' => Some((Symbol::Tilde, 1)),
            '!' => Some((Symbol::Bang, 1)),
            '%' => Some((Symbol::Percent, 1)),
            '<' => Some((Symbol::LessThan, 1)),
            '>' => Some((Symbol::GreaterThan, 1)),
            '&' => Some((Symbol::Ampersand, 1)),
            '^' => Some((Symbol::Caret, 1)),
            '|' => Some((Symbol::Pipe, 1)),
            '(' => Some((Symbol::LParen, 1)),
            ')' => Some((Symbol::RParen, 1)),
            '{' => Some((Symbol::LBrace, 1)),
            '}' => Some((Symbol::RBrace, 1)),
            _ => None
        }
    }
}

impl Symbol {
    pub fn infix_binding_power(&self) -> Option<(u8, u8)> {
        match self {
            Symbol::Star => Some((60, 61)),
            Symbol::Slash => Some((60, 61)),
            Symbol::Percent => Some((60, 61)),
            Symbol::Plus => Some((50, 51)),
            Symbol::Minus => Some((50, 51)),
            Symbol::LessThan => Some((40, 41)),
            Symbol::GreaterThan => Some((40, 41)),
            Symbol::Ampersand => Some((30, 31)),
            Symbol::Caret => Some((25, 26)),
            Symbol::Tilde => Some((20, 21)),
            _ => None,
        }
    }

    pub fn prefix_binding_power(&self) -> Option<u8> {
        match self {
            Symbol::Minus => Some(70),
            Symbol::Tilde => Some(70),
            Symbol::Bang => Some(70),
            _ => None,
        }
    }

    pub fn to_binary_operator(&self) -> Option<BinaryOp> {
        match self {
            Symbol::Slash => { Some(BinaryOp::Div) }
            Symbol::Star => { Some(BinaryOp::Mul) }
            Symbol::Plus => { Some(BinaryOp::Add) }
            Symbol::Minus => { Some(BinaryOp::Sub) }
            Symbol::Percent => { Some(BinaryOp::Mod) }
            Symbol::LessThan => { Some(BinaryOp::Lt) }
            Symbol::GreaterThan => { Some(BinaryOp::Gt) }
            Symbol::Ampersand => { Some(BinaryOp::And) }
            Symbol::Caret => { Some(BinaryOp::Xor) }
            Symbol::Pipe => { Some(BinaryOp::Or) }
            _ => None
        }
    }
}

#[derive(Debug, Clone)]
pub enum CompoundSymbol {
    LineComment,
    DoubleAsterisk,
    BitShiftLeft,
    BitShiftRight,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equals,
    NotEquals,
    And,
    Or
}

static LINE_COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^//[^\r\n]*").unwrap());

impl TokenTrait for CompoundSymbol {
    fn try_parse(input: &str) -> Option<(Self, usize)> {
        if let Some(m) = LINE_COMMENT.find(input) {
            return Some((CompoundSymbol::LineComment, m.len()));
        }

        const TOKENS: &[(&str, CompoundSymbol)] = &[
            ("**", CompoundSymbol::DoubleAsterisk),
            ("<<", CompoundSymbol::BitShiftLeft),
            (">>", CompoundSymbol::BitShiftRight),
            ("<=", CompoundSymbol::LessThanOrEqual),
            (">=", CompoundSymbol::GreaterThanOrEqual),
            ("==", CompoundSymbol::Equals),
            ("!=", CompoundSymbol::NotEquals),
            ("&&", CompoundSymbol::And),
            ("||", CompoundSymbol::Or),
        ];

        TOKENS.iter().find_map(|(pat, variant)| {
            input.starts_with(pat).then(|| (variant.clone(), pat.len()))
        })
    }
}

impl CompoundSymbol {
    pub fn infix_binding_power(&self) -> Option<(u8, u8)> {
        match self {
            CompoundSymbol::DoubleAsterisk => Some((80, 79)),
            CompoundSymbol::BitShiftLeft => Some((45, 46)),
            CompoundSymbol::BitShiftRight => Some((45, 46)),
            CompoundSymbol::LessThanOrEqual => Some((40, 41)),
            CompoundSymbol::GreaterThanOrEqual => Some((40, 41)),
            CompoundSymbol::Equals => Some((35, 36)),
            CompoundSymbol::NotEquals => Some((35, 36)),
            CompoundSymbol::And => Some((15, 16)),
            CompoundSymbol::Or => Some((10, 11)),
            _ => None
        }
    }

    pub fn to_binary_operator(&self) -> Option<BinaryOp> {
        match self {
            CompoundSymbol::DoubleAsterisk => Some(BinaryOp::Pow),
            CompoundSymbol::BitShiftLeft => Some(BinaryOp::Shl),
            CompoundSymbol::BitShiftRight => Some(BinaryOp::Shr),
            CompoundSymbol::LessThanOrEqual => Some(BinaryOp::Leq),
            CompoundSymbol::GreaterThanOrEqual => Some(BinaryOp::Geq),
            CompoundSymbol::Equals => Some(BinaryOp::Eq),
            CompoundSymbol::NotEquals => Some(BinaryOp::Neq),
            CompoundSymbol::And => Some(BinaryOp::LogicAnd),
            CompoundSymbol::Or => Some(BinaryOp::LogicOr),
            _ => None
        }
    }
}