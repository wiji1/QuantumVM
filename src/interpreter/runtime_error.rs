use crate::parser::parse_error::ParseError;

#[derive(Debug)]
pub enum RuntimeError {
    UnsupportedOperation(String),
    TypeMismatch(String),
    UndefinedVariable(String),
    NullPointer,
    InvalidSize,
    UnsupportedType,
    ConstReassignment(String),
    DuplicateVariable(String),
    IndexOutOfBounds(usize, usize),
    InvalidArgCount(usize, usize),
    UndefinedFunction(String),
    InvalidCall(String),
    InvalidControlFlow,
    FileNotFound(String),
    ParseError(ParseError),
    InvalidQubitAccess(String),
    NoStateVector,
    RecursionLimit,
}