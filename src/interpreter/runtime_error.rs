#[derive(Debug)]
pub enum RuntimeError {
    UnsupportedOperation(String),
    TypeMismatch(String),
    UndefinedVariable(String),
    NullPointer,
    InvalidSize,
    UnsupportedType,
    ConstAssignment(String),
    IndexOutOfBounds(usize, usize),
    InvalidArgCount(usize, usize),
    UndefinedFunction(String),
    InvalidCall(String),
    InvalidControlFlow,
}