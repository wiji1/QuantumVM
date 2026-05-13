use crate::interpreter::runtime_error::RuntimeError;
use crate::interpreter::value::Value;
use crate::lexer::type_def::TypeDefinition;
use crate::parser::expression::Expr;

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
    Bool(Option<Expr>),
    Duration(Option<Expr>),
    Complex(Option<Box<ClassicalType>>),
}

impl ClassicalType {
    pub fn get_default_value(&self, size: Option<Expr>) -> Result<Value, RuntimeError> {
        match self {
            ClassicalType::Bit(_) => {
                let width = match size {
                    Some(Expr::Int(n)) => n as usize,
                    None => 1,
                    _ => return Err(RuntimeError::InvalidSize),
                };

                Ok(Value::Bits { value: 0, width })
            },
            ClassicalType::Int(_) => Ok(Value::Int(0)),
            ClassicalType::UInt(_) => Ok(Value::Int(0)),
            ClassicalType::Float(_) => Ok(Value::Float(0.0)),
            ClassicalType::Bool(_) => Ok(Value::Bool(false)),
            _ =>  Err(RuntimeError::UnsupportedType),
        }
    }
}