use crate::interpreter::runtime_error::{RuntimeError, RuntimeErrorKind};
use crate::interpreter::value::Value;
use crate::parser::expression::{Expr, ExprKind};
use crate::parser::statement::Stmt;

#[derive(Debug, Clone)]
pub enum UnaryOp  { Neg, BitNot, LogicNot }

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Pow,
    And, Or, Xor, Shl, Shr,
    LogicAnd, LogicOr,
    Eq, Neq, Lt, Gt, Leq, Geq,
    Concat,
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
pub enum ParamType {
    Scalar(ClassicalType),
    Qubit(Option<Expr>),
    ArrayRef {
        mutable: bool,
        element_ty: ClassicalType,
        dimensions: ArrayDimensions,
    },
    Unspecified
}

#[derive(Debug, Clone)]
pub enum ArrayDimensions {
    Explicit(Vec<Expr>),
    Dim(Expr),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub ty: ParamType,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub values: Vec<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum IoDirection {
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub enum GateModifier {
    Inv,
    Pow(Expr),
    Ctrl(Option<Expr>),
    NegCtrl(Option<Expr>),
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
    Stretch(Option<Expr>),
    Complex(Option<Box<ClassicalType>>),
}

impl ClassicalType {
    pub fn get_default_value(&self, size: Option<Expr>) -> Result<Value, RuntimeError> {
        match self {
            ClassicalType::Bit(_) => {
                let width = match size {
                    Some(ref expr) => match &expr.kind {
                        ExprKind::Int(n) => *n as usize,
                        _ => return Err(RuntimeError::new(RuntimeErrorKind::InvalidSize)),
                    },
                    None => 1,
                };

                Ok(Value::Bits { value: 0, width })
            },
            ClassicalType::Int(_) => Ok(Value::Int(0)),
            ClassicalType::UInt(_) => Ok(Value::Int(0)),
            ClassicalType::Float(_) => Ok(Value::Float(0.0)),
            ClassicalType::Bool(_) => Ok(Value::Bool(false)),
            ClassicalType::Angle(_) => Ok(Value::Float(0.0)),
            ClassicalType::Complex(_) => Ok(Value::Complex(0.0, 0.0)),
            ClassicalType::Duration(_) => Ok(Value::Timing("".to_string())),
            ClassicalType::Stretch(_) => Ok(Value::Timing("".to_string())),
            _ =>  Err(RuntimeError::new(RuntimeErrorKind::UnsupportedType)),
        }
    }

    pub fn get_size(&self) -> Option<Expr> {
        match self {
            ClassicalType::Bit(s)
            | ClassicalType::Int(s)
            | ClassicalType::UInt(s)
            | ClassicalType::Float(s)
            | ClassicalType::Angle(s)
            | ClassicalType::Bool(s)
            | ClassicalType::Duration(s)
            | ClassicalType::Stretch(s) => s.clone(),

            ClassicalType::Complex(_) => None,
        }
    }
}