use crate::enums::expression::Expr;

#[derive(Debug, Clone)]
pub enum UnaryOp  { Neg, BitNot, LogicNot }

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Pow,
    And, Or, Xor, Shl, Shr,
    LogicAnd, LogicOr,
    Eq, Neq, Lt, Gt, Leq, Geq,
}

#[derive(Debug, Clone)]
pub enum AssignOp {
    Eq,
    Compound(BinaryOp),
}

#[derive(Debug, Clone)]
pub enum ForIter {
    Set(Vec<Expr>),
    Range {
        start: Option<Box<Expr>>,
        stop: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
    },
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum GateOperand {
    Ident(IndexedIdent),
    HardwareQubit(u32),
}

#[derive(Debug, Clone)]
pub struct IndexedIdent {
    pub name: String,
    pub indices: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum ClassicalType {
    Bit(Option<Expr>),
    Int(Option<Expr>),
    UInt(Option<Expr>),
    Float(Option<Expr>),
    Angle(Option<Expr>),
    Bool,
    Duration,
    Complex(Option<Box<ClassicalType>>),
}