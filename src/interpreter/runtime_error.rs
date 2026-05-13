#[derive(Debug)]
pub enum RuntimeError {
    UnsupportedOperation(String),
    TypeMismatch(String),
    UndefinedVariable(String),
    NullPointer,
    InvalidSize,
    UnsupportedType,
}