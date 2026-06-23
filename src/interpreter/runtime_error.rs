use crate::parser::parse_error::ParseError;
use crate::lexer::Span;
use std::fmt;

#[derive(Debug, Clone)]
pub enum RuntimeErrorKind {
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

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub span: Option<Span>,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind) -> Self {
        Self { kind, span: None }
    }

    pub fn with_span(kind: RuntimeErrorKind, span: Span) -> Self {
        Self { kind, span: Some(span) }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RuntimeErrorKind::*;
        match &self.kind {
            UnsupportedOperation(op) => write!(f, "Unsupported operation: {}", op),
            TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            UndefinedVariable(name) => write!(f, "Undefined variable: '{}'", name),
            NullPointer => write!(f, "Null pointer dereference"),
            InvalidSize => write!(f, "Invalid size"),
            UnsupportedType => write!(f, "Unsupported type"),
            ConstReassignment(name) => write!(f, "Cannot reassign constant: '{}'", name),
            DuplicateVariable(name) => write!(f, "Variable '{}' is already defined", name),
            DuplicateFunction(name) => write!(f, "Function '{}' is already defined", name),
            DuplicateGate(name) => write!(f, "Gate '{}' is already defined", name),
            IndexOutOfBounds(idx, len) => {
                if *len == 0 { write!(f, "Index {} out of bounds (array is empty)", idx) }
                else { write!(f, "Index {} out of bounds (valid range: 0 to {})", idx, len - 1) }
            },
            InvalidIndex => write!(f, "Invalid index"),
            InvalidArgCount(expected, found) => write!(f, "Expected {} argument(s), found {}", expected, found),
            UndefinedFunction(name) => write!(f, "Undefined function: '{}'", name),
            InvalidCall(msg) => write!(f, "Invalid function call: {}", msg),
            InvalidControlFlow => write!(f, "Invalid control flow (break/continue outside loop)"),
            FileNotFound(path) => write!(f, "File not found: '{}'", path),
            ParseError(err) => write!(f, "Parse error: {:?}", err),
            InvalidQubitAccess(msg) => write!(f, "Invalid qubit access: {}", msg),
            DivideByZero => write!(f, "Division by zero"),
            NoStateVector => write!(f, "No quantum state vector initialized"),
            RecursionLimit => write!(f, "Recursion limit exceeded"),
            SelfConcatenation(name) => write!(f, "Cannot concatenate variable '{}' to itself", name),
        }
    }
}