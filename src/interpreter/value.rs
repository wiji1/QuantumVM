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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;

        match (self, other) {
            (Int(a), Int(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (
                Bits {
                    value: av,
                    width: aw,
                },
                Bits {
                    value: bv,
                    width: bw,
                },
            ) => aw == bw && av == bv,
            (Complex(ar, ai), Complex(br, bi)) => {
                ar == br && ai == bi
            }
            (Qubit(a), Qubit(b)) => a == b,
            (Void, Void) => true,
            (Array(a), Array(b)) => a == b,
            (Int(a), Float(b)) | (Float(b), Int(a)) => (*a as f64) == *b,
            (Bool(a), Int(b)) | (Int(b), Bool(a)) => {
                (*a as i64) == *b
            }
            (Bool(a), Float(b)) | (Float(b), Bool(a)) => {
                (*a as u8 as f64) == *b
            }
            (
                Bits {
                    value,
                    width: 1,
                },
                Bool(b),
            )
            | (
                Bool(b),
                Bits {
                    value,
                    width: 1,
                },
            ) => (*value != 0) == *b,
            _ => false,
        }
    }
}