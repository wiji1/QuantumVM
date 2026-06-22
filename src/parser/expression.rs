use std::fmt::Formatter;
use crate::lexer::Span;
use crate::parser::supporting_types::{BinaryOp, ClassicalType, GateOperand, IndexedIdent, UnaryOp};

#[derive(Debug, Clone)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    Imaginary(f64),
    Timing(String),
    Bits(u64, usize),
    Array(Box<Vec<Expr>>),

    Ident(String),
    IndexedIdent(IndexedIdent),

    Measure(Box<GateOperand>),

    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { op: BinaryOp, lhs: Box<Expr>, rhs: Box<Expr> },

    Call { name: String, args: Vec<Expr> },
    Cast { ty: Box<ClassicalType>, expr: Box<Expr> },
    Range { start: Option<Box<Expr>>, stop: Option<Box<Expr>>, step: Option<Box<Expr>> },
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}