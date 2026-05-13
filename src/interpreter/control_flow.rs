use crate::interpreter::value::Value;

pub enum ControlFlow {
    None,
    Break,
    Continue,
    Return(Value),
}