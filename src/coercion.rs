use crate::interpreter::value::Value;
use crate::type_checker::type_repr::Type;
use std::fmt;

#[derive(Debug, Clone)]
pub enum CoercionError {
    InvalidCoercion { from: String, to: String },
    ValueOutOfRange { value: String, target: String },
}

impl fmt::Display for CoercionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoercionError::InvalidCoercion { from, to } => {
                write!(f, "Cannot coerce from {} to {}", from, to)
            }
            CoercionError::ValueOutOfRange { value, target } => {
                write!(f, "Value {} is out of range for type {}", value, target)
            }
        }
    }
}

pub fn can_type_coerce(from: &Type, to: &Type) -> bool {
    if from.is_compatible_with(to) { return true; }

    match (from, to) {
        (Type::Bool, Type::Int(_)) | (Type::Bool, Type::UInt(_)) => true,
        (Type::Bool, Type::Float(_)) => true,

        (Type::Bit(Some(1)), Type::Bool) | (Type::Bool, Type::Bit(Some(1))) => true,
        (Type::Bit(None), Type::Bool) | (Type::Bool, Type::Bit(None)) => true,

        (Type::Bit(b), Type::Int(_)) | (Type::Bit(b), Type::UInt(_)) => {
            match b {
                Some(1) => true,
                _ => false
            }
        },
        (Type::Int(_), Type::Bit(_)) | (Type::UInt(_), Type::Bit(_)) => true,

        (Type::Int(_), Type::Float(_)) | (Type::UInt(_), Type::Float(_)) => true,

        (Type::Int(_), Type::Complex(_)) | (Type::UInt(_), Type::Complex(_)) => true,
        (Type::Float(_), Type::Complex(_)) | (Type::Bool, Type::Complex(_)) => true,

        (Type::Angle(_), Type::Float(_)) | (Type::Float(_), Type::Angle(_)) => true,

        (Type::Int(_), Type::Angle(_)) | (Type::UInt(_), Type::Angle(_)) => true,
        (Type::Bit(a), Type::Bit(b)) => match (a, b) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(a), Some(b)) => a <= b,
        },
        _ => false,
    }
}

pub fn coerce_value(value: Value, target: &Type) -> Result<Value, CoercionError> {
    let from_type = infer_type_from_value(&value);

    if !can_type_coerce(&from_type, target) {
        return Err(CoercionError::InvalidCoercion {
            from: from_type.display_name(),
            to: target.display_name(),
        });
    }

    match (value, target) {
        (Value::Int(i), Type::Int(_)) => Ok(Value::Int(i)),
        (Value::Int(i), Type::UInt(_)) => Ok(Value::Int(i)),
        (Value::Float(f), Type::Float(_)) => Ok(Value::Float(f)),
        (Value::Bool(b), Type::Bool) => Ok(Value::Bool(b)),
        (Value::Complex(re, im), Type::Complex(_)) => Ok(Value::Complex(re, im)),
        (val @ Value::Bits { .. }, Type::Bit(_)) => Ok(val),
        (Value::Timing(s), Type::Duration) => Ok(Value::Timing(s)),

        (Value::Bool(b), Type::Int(_)) => Ok(Value::Int(b as i64)),
        (Value::Bool(b), Type::UInt(_)) => Ok(Value::Int(b as i64)),

        (Value::Bool(b), Type::Float(_)) => Ok(Value::Float(b as i64 as f64)),

        (Value::Bool(b), Type::Bit(Some(1))) | (Value::Bool(b), Type::Bit(None)) => {
            Ok(Value::Bits { value: b as u64, width: 1 })
        }

        (Value::Bits { value, width: 1 }, Type::Bool) => {
            Ok(Value::Bool(value != 0))
        }

        (Value::Bits { value, .. }, Type::Int(_)) => Ok(Value::Int(value as i64)),
        (Value::Bits { value, .. }, Type::UInt(_)) => Ok(Value::Int(value as i64)),

        (Value::Bits { value, width }, Type::Bool) => {
            if width == 1 {
                Ok(Value::Bool(value != 0))
            } else {
                Err(CoercionError::InvalidCoercion {
                    from: format!("bit[{}]", width),
                    to: "bool".to_string(),
                })
            }
        }

        (Value::Int(i), Type::Bit(size)) => {
            let width = match size {
                Some(w) => *w as usize,
                None => 1,
            };
            Ok(Value::Bits { value: i as u64, width })
        }

        (Value::Int(i), Type::Float(_)) => Ok(Value::Float(i as f64)),

        (Value::Float(f), Type::Int(_)) => Ok(Value::Int(f as i64)),

        (Value::Int(i), Type::Complex(_)) => Ok(Value::Complex(i as f64, 0.0)),

        (Value::Float(f), Type::Angle(_)) => Ok(Value::Float(f)),

        (Value::Int(i), Type::Angle(_)) => Ok(Value::Float(i as f64)),

        (Value::Float(f), Type::Complex(_)) => Ok(Value::Complex(f, 0.0)),

        (Value::Bool(b), Type::Complex(_)) => Ok(Value::Complex(b as i64 as f64, 0.0)),

        (val, target_ty) => {
            Err(CoercionError::InvalidCoercion {
                from: infer_type_from_value(&val).display_name(),
                to: target_ty.display_name(),
            })
        }
    }
}

pub fn coerce_to_bool(value: Value) -> Result<Value, CoercionError> {
    match value {
        Value::Bool(b) => Ok(Value::Bool(b)),
        Value::Int(i) => Ok(Value::Bool(i != 0)),
        Value::Float(f) => Ok(Value::Bool(f != 0.0)),
        Value::Bits { value, width: 1 } => Ok(Value::Bool(value != 0)),
        Value::Bits { value, width } => {
            Ok(Value::Bool(value != 0))
        }
        _ => Err(CoercionError::InvalidCoercion {
            from: infer_type_from_value(&value).display_name(),
            to: "bool".to_string(),
        }),
    }
}

pub fn cast_value(value: Value, target: &Type) -> Result<Value, CoercionError> {
    match (value, target) {
        (Value::Int(i), Type::Int(_) | Type::UInt(_)) => Ok(Value::Int(i)),
        (Value::Float(f), Type::Float(_)) => Ok(Value::Float(f)),
        (Value::Bool(b), Type::Bool) => Ok(Value::Bool(b)),
        (Value::Complex(re, im), Type::Complex(_)) => Ok(Value::Complex(re, im)),
        (Value::Timing(s), Type::Duration) => Ok(Value::Timing(s)),

        (Value::Float(f), Type::Int(_) | Type::UInt(_)) => Ok(Value::Int(f as i64)),
        (Value::Bool(b), Type::Int(_) | Type::UInt(_)) => Ok(Value::Int(b as i64)),
        (Value::Bits { value, .. }, Type::Int(_) | Type::UInt(_)) => Ok(Value::Int(value as i64)),

        (Value::Int(i), Type::Float(_)) => Ok(Value::Float(i as f64)),
        (Value::Bool(b), Type::Float(_)) => Ok(Value::Float(b as i64 as f64)),

        (Value::Int(i), Type::Bool) => Ok(Value::Bool(i != 0)),
        (Value::Float(f), Type::Bool) => Ok(Value::Bool(f != 0.0)),
        (Value::Bits { value, .. }, Type::Bool) => Ok(Value::Bool(value != 0)),

        (Value::Int(i), Type::Bit(size)) => {
            let width = match size {
                Some(w) => *w as usize,
                None => 1,
            };
            Ok(Value::Bits { value: i as u64, width })
        }
        (Value::Bool(b), Type::Bit(_)) => {
            Ok(Value::Bits { value: b as u64, width: 1 })
        }
        (Value::Bits { value, .. }, Type::Bit(size)) => {
            let width = match size {
                Some(w) => *w as usize,
                None => 1,
            };
            Ok(Value::Bits { value, width })
        }

        (Value::Int(i), Type::Complex(_)) => Ok(Value::Complex(i as f64, 0.0)),
        (Value::Float(f), Type::Complex(_)) => Ok(Value::Complex(f, 0.0)),
        (Value::Bool(b), Type::Complex(_)) => Ok(Value::Complex(b as i64 as f64, 0.0)),

        (Value::Float(f), Type::Angle(_)) => Ok(Value::Float(f)),
        (Value::Int(i), Type::Angle(_)) => Ok(Value::Float(i as f64)),

        (val, target_ty) => {
            Err(CoercionError::InvalidCoercion {
                from: infer_type_from_value(&val).display_name(),
                to: target_ty.display_name(),
            })
        }
    }
}

pub fn infer_type_from_value(value: &Value) -> Type {
    match value {
        Value::Int(_) => Type::Int(None),
        Value::Float(_) => Type::Float(None),
        Value::Bool(_) => Type::Bool,
        Value::Bits { width, .. } => Type::Bit(Some(*width as i64)),
        Value::Complex(_, _) => Type::Complex(Box::new(Type::Float(None))),
        Value::Qubit(indices) => Type::Qubit(Some(indices.len() as i64)),
        Value::Void => Type::Void,
        Value::Array(elements) => {
            if elements.is_empty() {
                Type::Array {
                    element_type: Box::new(Type::Unknown),
                    dimensions: vec![Some(0)],
                }
            } else {
                let elem_type = infer_type_from_value(&elements[0]);
                Type::Array {
                    element_type: Box::new(elem_type),
                    dimensions: vec![Some(elements.len() as i64)],
                }
            }
        }
        Value::Range { .. } => Type::Range,
        Value::Timing(_) => Type::Duration,
    }
}
