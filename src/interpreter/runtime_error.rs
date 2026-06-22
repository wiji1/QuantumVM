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
    DuplicateFunction(String),
    DuplicateGate(String),
    IndexOutOfBounds(usize, usize),
    InvalidIndex,
    InvalidArgCount(usize, usize),
    UndefinedFunction(String),
    InvalidCall(String),
    InvalidControlFlow,
    FileNotFound(String),
    ParseError(ParseError),
    InvalidQubitAccess(String),
    DivideByZero,
    NoStateVector,
    RecursionLimit,
    SelfConcatenation(String),
}