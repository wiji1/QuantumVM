use crate::type_checker::type_repr::Type;
use std::fmt;

#[derive(Debug, Clone)]
pub enum StaticError {
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

    DuplicateVariableDefinition {
        name: String
    },

    Other {
        message: String,
    },
}

impl fmt::Display for StaticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StaticError::TypeMismatch { expected, found, context } => {
                write!(
                    f,
                    "Type mismatch in {context}: expected {}, found {}",
                    expected.display_name(),
                    found.display_name()
                )
            }
            StaticError::UndefinedVariable { name } => {
                write!(f, "Undefined variable: {name}")
            }
            StaticError::UndefinedFunction { name } => {
                write!(f, "Undefined function: {name}")
            }
            StaticError::UndefinedGate { name } => {
                write!(f, "Undefined gate: {name}")
            }
            StaticError::ArityMismatch { name, expected, found } => {
                write!(
                    f,
                    "Function {name} expects {expected} argument(s), but {found} were provided"
                )
            }
            StaticError::InvalidCast { from, to } => {
                write!(
                    f,
                    "Invalid cast from {} to {}",
                    from.display_name(),
                    to.display_name()
                )
            }
            StaticError::AssignmentToConst { name } => {
                write!(f, "Cannot assign to constant: {name}")
            }
            StaticError::InvalidOperation { op, type_ } => {
                write!(
                    f,
                    "Invalid operation {op} for type {}",
                    type_.display_name()
                )
            }
            StaticError::CannotInferType { name } => {
                write!(f, "Cannot infer type for: {name}")
            }
            StaticError::NonIntegerIndex { index_type } => {
                write!(
                    f,
                    "Array index must be an integer, found {}",
                    index_type.display_name()
                )
            }
            StaticError::DimensionMismatch { expected, found } => {
                write!(
                    f,
                    "Dimension mismatch: expected {expected} dimension(s), found {found}"
                )
            }
            StaticError::InvalidArrayAccess { type_ } => {
                write!(
                    f,
                    "Cannot index type {}",
                    type_.display_name()
                )
            }
            StaticError::NonBooleanCondition { found } => {
                write!(
                    f,
                    "Condition must be boolean, found {}",
                    found.display_name()
                )
            }
            StaticError::ReturnTypeMismatch { expected, found } => {
                write!(
                    f,
                    "Return type mismatch: expected {}, found {}",
                    expected.display_name(),
                    found.display_name()
                )
            }
            StaticError::MissingReturn { function } => {
                write!(f, "Function {function} is missing a return statement")
            }
            StaticError::BinaryOpTypeMismatch { op, lhs, rhs } => {
                write!(
                    f,
                    "Cannot apply operator {op} to types {} and {}",
                    lhs.display_name(),
                    rhs.display_name()
                )
            }
            StaticError::UnaryOpTypeMismatch { op, operand } => {
                write!(
                    f,
                    "Cannot apply operator {op} to type {}",
                    operand.display_name()
                )
            }
            StaticError::QuantumOpOnClassical { name } => {
                write!(f, "Cannot apply quantum operation to classical variable: {name}")
            }
            StaticError::ClassicalOpOnQuantum { name } => {
                write!(f, "Cannot apply classical operation to quantum variable: {name}")
            }
            StaticError::DuplicateVariableDefinition { name } => {
                write!(f, "Duplicate variable definition: {name}")
            }
            StaticError::Other { message } => {
                write!(f, "{message}")
            }
        }
    }
}

impl std::error::Error for StaticError {}
