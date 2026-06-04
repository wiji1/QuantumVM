use {ClassicalType, ParamType};
use crate::parser::expression::Expr;
use crate::parser::supporting_types::*;
use Type::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int(Option<i64>),
    UInt(Option<i64>),
    Float(Option<i64>),
    Bool,
    Bit(Option<i64>),
    Angle(Option<i64>),
    Duration,
    Stretch,
    Complex(Box<Type>),

    Qubit(Option<i64>),

    Array {
        element_type: Box<Type>,
        dimensions: Vec<Option<i64>>,
    },

    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },

    Void,
    Range,
    Unspecified,
    Unknown,
}

impl Type {
    pub fn from_classical_type(ct: &ClassicalType) -> Self {
        match ct {
            ClassicalType::Bit(size) => Type::Bit(Self::extract_size(size)),
            ClassicalType::Int(size) => Type::Int(Self::extract_size(size)),
            ClassicalType::UInt(size) => Type::UInt(Self::extract_size(size)),
            ClassicalType::Float(size) => Type::Float(Self::extract_size(size)),
            ClassicalType::Angle(size) => Type::Angle(Self::extract_size(size)),
            ClassicalType::Bool(_) => Type::Bool,
            ClassicalType::Duration(_) => Type::Duration,
            ClassicalType::Stretch(_) => Type::Stretch,
            ClassicalType::Complex(inner) => {
                match inner {
                    Some(inner_ty) => Type::Complex(Box::new(Type::from_classical_type(inner_ty))),
                    None => Type::Complex(Box::new(Type::Float(None))),
                }
            }
        }
    }

    pub fn from_param_type(pt: &ParamType) -> Self {
        match pt {
            ParamType::Scalar(ct) => Type::from_classical_type(ct),
            ParamType::Qubit(size) => Type::Qubit(Self::extract_size(size)),
            ParamType::ArrayRef { element_ty, dimensions, .. } => {
                let elem_type = Type::from_classical_type(element_ty);
                let dims = match dimensions {
                    ArrayDimensions::Explicit(exprs) => {
                        exprs.iter().map(|e| Self::extract_size(&Some(e.clone()))).collect()
                    }
                    ArrayDimensions::Dim(expr) => {
                        vec![Self::extract_size(&Some(expr.clone()))]
                    }
                };
                Type::Array {
                    element_type: Box::new(elem_type),
                    dimensions: dims,
                }
            }
            ParamType::Unspecified => Type::Unspecified,
        }
    }

    fn extract_size(size: &Option<Expr>) -> Option<i64> {
        match size {
            Some(Expr::Int(n)) => Some(*n),
            _ => None,
        }
    }

    pub fn is_compatible_with(&self, other: &Type) -> bool {
        if self == other { return true; }

        if matches!(self, Unspecified | Unknown) || matches!(other, Unspecified | Unknown) {
            return true;
        }

        match (self, other) {
            (Int(a), Int(b)) | (UInt(a), UInt(b)) | (Float(a), Float(b)) |
            (Bit(a), Bit(b)) | (Angle(a), Angle(b)) | (Qubit(a), Qubit(b)) => {
                match (a, b) {
                    (None, _) | (_, None) => true,
                    (Some(x), Some(y)) => x == y,
                }
            }

            (Int(_), UInt(_)) | (UInt(_), Int(_)) => true,

            (Bool, Bit(Some(1))) | (Bit(Some(1)), Bool) => true,

            (Complex(a), Complex(b)) => a.is_compatible_with(b),

            (Array { element_type: e1, dimensions: d1 },
             Array { element_type: e2, dimensions: d2 }) => {
                e1.is_compatible_with(e2) && d1.len() == d2.len()
            }

            (Function { params: p1, return_type: r1 },
             Function { params: p2, return_type: r2 }) => {
                p1.len() == p2.len() &&
                p1.iter().zip(p2.iter()).all(|(a, b)| a.is_compatible_with(b)) &&
                r1.is_compatible_with(r2)
            }

            _ => false,
        }
    }

    pub fn can_coerce_to(&self, target: &Type) -> bool {
        if self.is_compatible_with(target) { return true; }

        match (self, target) {
            (Bool, Int(_)) | (Bool, UInt(_)) => true,
            (Bool, Float(_)) | (Int(_), Float(_)) | (UInt(_), Float(_)) => true,

            (Bit(Some(1)), Bool) | (Bool, Bit(Some(1))) => true,

            (Bit(_), Int(_)) | (Bit(_), UInt(_)) => true,

            (Int(_), Complex(_)) | (UInt(_), Complex(_)) |
            (Float(_), Complex(_)) | (Bool, Complex(_)) => true,

            (Angle(_), Float(_)) => true,

            _ => false,
        }
    }

    pub fn coerce_to(&self, target: &Type) -> Option<Type> {
        if self.can_coerce_to(target) { Some(target.clone()) }
        else { None }
    }

    pub fn binary_result_type(&self, other: &Type, op: &BinaryOp) -> Option<Type> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Pow => {
                if matches!(self, Complex(_)) || matches!(other, Complex(_)) {
                    return Some(Complex(Box::new(Float(None))));
                }

                if matches!(self, Float(_)) || matches!(other, Float(_)) {
                    return Some(Float(None));
                }

                if (matches!(self, Int(_) | UInt(_)) && matches!(other, Int(_) | UInt(_))) {
                    return Some(Int(None));
                }

                None
            }

            BinaryOp::Mod => {
                if matches!(self, Int(_) | UInt(_)) && matches!(other, Int(_) | UInt(_)) {
                    Some(Int(None))
                } else { None }
            }

            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor | BinaryOp::Shl | BinaryOp::Shr => {
                match (self, other) {
                    (Bit(a), Bit(b)) => {
                        match (a, b) {
                            (Some(x), Some(y)) if x == y => Some(Bit(Some(*x))),
                            (None, _) | (_, None) => Some(Bit(None)),
                            _ => None,
                        }
                    }
                    (Int(_), Int(_)) | (UInt(_), UInt(_)) | (Int(_), UInt(_)) | (UInt(_), Int(_)) => {
                        Some(Int(None))
                    }
                    (Bool, Bool) if matches!(op, BinaryOp::And | BinaryOp::Or | BinaryOp::Xor) => {
                        Some(Bool)
                    }
                    _ => None,
                }
            }

            BinaryOp::LogicAnd | BinaryOp::LogicOr => {
                if matches!(self, Bool) && matches!(other, Bool) { Some(Bool) }
                else { None }
            }

            BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Gt |
            BinaryOp::Leq | BinaryOp::Geq => {
                let self_numeric = matches!(self, Int(_) | UInt(_) | Float(_) | Bool | Angle(_));
                let other_numeric = matches!(other, Int(_) | UInt(_) | Float(_) | Bool | Angle(_));

                if self_numeric && other_numeric { Some(Bool) }
                else if matches!(self, Bool) && matches!(other, Bool) { Some(Bool) }
                else { None }
            }
        }
    }

    pub fn unary_result_type(&self, op: &UnaryOp) -> Option<Type> {

        match op {
            UnaryOp::Neg => {
                match self {
                    Int(_) | UInt(_) | Float(_) | Complex(_) => Some(self.clone()),
                    _ => None,
                }
            }
            UnaryOp::BitNot => {
                match self {
                    Bit(_) | Int(_) | UInt(_) => Some(self.clone()),
                    _ => None,
                }
            }
            UnaryOp::LogicNot => {
                match self {
                    Bool => Some(Bool),
                    _ => None,
                }
            }
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int(_) | Type::UInt(_) | Type::Float(_) | Type::Angle(_) | Type::Complex(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Int(_) | Type::UInt(_))
    }

    pub fn display_name(&self) -> String {
        match self {
            Int(Some(w)) => format!("int[{w}]"),
            Int(None) => "int".to_string(),
            UInt(Some(w)) => format!("uint[{w}]"),
            UInt(None) => "uint".to_string(),
            Float(Some(w)) => format!("float[{w}]"),
            Float(None) => "float".to_string(),
            Bool => "bool".to_string(),
            Bit(Some(w)) => format!("bit[{w}]"),
            Bit(None) => "bit".to_string(),
            Angle(Some(w)) => format!("angle[{w}]"),
            Angle(None) => "angle".to_string(),
            Duration => "duration".to_string(),
            Stretch => "stretch".to_string(),
            Complex(inner) => format!("complex[{}]", inner.display_name()),
            Qubit(Some(w)) => format!("qubit[{w}]"),
            Qubit(None) => "qubit".to_string(),
            Array { element_type, dimensions } => {
                let dims = dimensions.iter()
                    .map(|d| match d {
                        Some(n) => n.to_string(),
                        None => "?".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}[{dims}]", element_type.display_name())
            }
            Function { params, return_type } => {
                let param_str = params.iter()
                    .map(|p| p.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({param_str}) -> {}", return_type.display_name())
            }
            Void => "void".to_string(),
            Range => "range".to_string(),
            Unspecified => "unspecified".to_string(),
            Unknown => "unknown".to_string(),
        }
    }
}