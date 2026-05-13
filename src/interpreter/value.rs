#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Bits { value: u64, width: usize },
    Complex(f64, f64),
    Qubit(usize),
    Void,
    Array(Vec<Value>),
}