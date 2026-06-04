use crate::type_checker::type_repr::Type;
use std::fmt;

#[derive(Debug, Clone)]
pub enum TypeError {
    TypeMismatch {
        expected: Type,
        found: Type,
        context: String,
    },

    UndefinedVariable {
        name: String,
    },

    UndefinedFunction {
        name: String,
    },

    UndefinedGate {
        name: String,
    },

    ArityMismatch {
        name: String,
        expected: usize,
        found: usize,
    },

    InvalidCast {
        from: Type,
        to: Type,
    },

    AssignmentToConst {
        name: String,
    },

    InvalidOperation {
        op: String,
        type_: Type,
    },

    CannotInferType {
        name: String,
    },

    NonIntegerIndex {
        index_type: Type,
    },

    DimensionMismatch {
        expected: usize,
        found: usize,
    },

    InvalidArrayAccess {
        type_: Type,
    },

    NonBooleanCondition {
        found: Type,
    },

    ReturnTypeMismatch {
        expected: Type,
        found: Type,
    },

    MissingReturn {
        function: String,
    },

    BinaryOpTypeMismatch {
        op: String,
        lhs: Type,
        rhs: Type,
    },

    UnaryOpTypeMismatch {
        op: String,
        operand: Type,
    },

    QuantumOpOnClassical {
        name: String,
    },

    ClassicalOpOnQuantum {
        name: String,
    },

    Other {
        message: String,
    },
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::TypeMismatch { expected, found, context } => {
                write!(
                    f,
                    "Type mismatch in {context}: expected {}, found {}",
                    expected.display_name(),
                    found.display_name()
                )
            }
            TypeError::UndefinedVariable { name } => {
                write!(f, "Undefined variable: {name}")
            }
            TypeError::UndefinedFunction { name } => {
                write!(f, "Undefined function: {name}")
            }
            TypeError::UndefinedGate { name } => {
                write!(f, "Undefined gate: {name}")
            }
            TypeError::ArityMismatch { name, expected, found } => {
                write!(
                    f,
                    "Function {name} expects {expected} argument(s), but {found} were provided"
                )
            }
            TypeError::InvalidCast { from, to } => {
                write!(
                    f,
                    "Invalid cast from {} to {}",
                    from.display_name(),
                    to.display_name()
                )
            }
            TypeError::AssignmentToConst { name } => {
                write!(f, "Cannot assign to constant: {name}")
            }
            TypeError::InvalidOperation { op, type_ } => {
                write!(
                    f,
                    "Invalid operation {op} for type {}",
                    type_.display_name()
                )
            }
            TypeError::CannotInferType { name } => {
                write!(f, "Cannot infer type for: {name}")
            }
            TypeError::NonIntegerIndex { index_type } => {
                write!(
                    f,
                    "Array index must be an integer, found {}",
                    index_type.display_name()
                )
            }
            TypeError::DimensionMismatch { expected, found } => {
                write!(
                    f,
                    "Dimension mismatch: expected {expected} dimension(s), found {found}"
                )
            }
            TypeError::InvalidArrayAccess { type_ } => {
                write!(
                    f,
                    "Cannot index type {}",
                    type_.display_name()
                )
            }
            TypeError::NonBooleanCondition { found } => {
                write!(
                    f,
                    "Condition must be boolean, found {}",
                    found.display_name()
                )
            }
            TypeError::ReturnTypeMismatch { expected, found } => {
                write!(
                    f,
                    "Return type mismatch: expected {}, found {}",
                    expected.display_name(),
                    found.display_name()
                )
            }
            TypeError::MissingReturn { function } => {
                write!(f, "Function {function} is missing a return statement")
            }
            TypeError::BinaryOpTypeMismatch { op, lhs, rhs } => {
                write!(
                    f,
                    "Cannot apply operator {op} to types {} and {}",
                    lhs.display_name(),
                    rhs.display_name()
                )
            }
            TypeError::UnaryOpTypeMismatch { op, operand } => {
                write!(
                    f,
                    "Cannot apply operator {op} to type {}",
                    operand.display_name()
                )
            }
            TypeError::QuantumOpOnClassical { name } => {
                write!(f, "Cannot apply quantum operation to classical variable: {name}")
            }
            TypeError::ClassicalOpOnQuantum { name } => {
                write!(f, "Cannot apply classical operation to quantum variable: {name}")
            }
            TypeError::Other { message } => {
                write!(f, "{message}")
            }
        }
    }
}

impl std::error::Error for TypeError {}
