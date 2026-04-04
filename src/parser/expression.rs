use crate::parser::supporting_types::{BinaryOp, ClassicalType, GateOperand, IndexedIdent, UnaryOp};

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    Imaginary(f64),
    Timing(String),

    Ident(String),
    IndexedIdent(IndexedIdent),

    Measure(Box<GateOperand>),

    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { op: BinaryOp, lhs: Box<Expr>, rhs: Box<Expr> },

    Index { expr: Box<Expr>, index: Box<Expr> },

    Call { name: String, args: Vec<Expr> },

    Cast { ty: Box<ClassicalType>, expr: Box<Expr> },

    Range { start: Option<Box<Expr>>, stop: Option<Box<Expr>>, step: Option<Box<Expr>> },
}