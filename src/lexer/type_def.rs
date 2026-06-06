use crate::interpreter::runtime_error::RuntimeError;
use crate::interpreter::value::Value;
use crate::lexer::{match_keyword, TokenTrait};
use crate::parser::expression::Expr;
use crate::parser::supporting_types::ClassicalType;

#[derive(Debug, Clone)]
pub enum TypeDefinition {
    Qubit,
    Bool,
    Bit,
    Int,
    UInt,
    Float,
    Angle,
    Complex,
    Duration,
    Array,
    Void,
    Stretch
}

//TODO: Make this compiler-enforced
impl TokenTrait for TypeDefinition {
    fn try_parse(input: &str) -> Option<(Self, usize)> {
        if let Some(len) = match_keyword(input, "qubit") {
            return Some((Self::Qubit, len));
        }
        if let Some(len) = match_keyword(input, "bool") {
            return Some((Self::Bool, len));
        }
        if let Some(len) = match_keyword(input, "bit") {
            return Some((Self::Bit, len));
        }
        if let Some(len) = match_keyword(input, "int") {
            return Some((Self::Int, len));
        }
        if let Some(len) = match_keyword(input, "uint") {
            return Some((Self::UInt, len));
        }
        if let Some(len) = match_keyword(input, "float") {
            return Some((Self::Float, len));
        }
        if let Some(len) = match_keyword(input, "angle") {
            return Some((Self::Angle, len));
        }
        if let Some(len) = match_keyword(input, "complex") {
            return Some((Self::Complex, len));
        }
        if let Some(len) = match_keyword(input, "duration") {
            return Some((Self::Duration, len));
        }
        if let Some(len) = match_keyword(input, "array") {
            return Some((Self::Array, len));
        }
        if let Some(len) = match_keyword(input, "void") {
            return Some((Self::Void, len));
        }
        if let Some(len) = match_keyword(input, "stretch") {
            return Some((Self::Stretch, len));
        }
        None
    }
}

impl TypeDefinition {
    pub fn get_classical_type(&self, size: Option<Expr>) -> Option<ClassicalType> {
        match self {
            TypeDefinition::Bool => Some(ClassicalType::Bool(size)),
            TypeDefinition::Bit => Some(ClassicalType::Bit(size)),
            TypeDefinition::Int => Some(ClassicalType::Int(size)),
            TypeDefinition::UInt => Some(ClassicalType::UInt(size)),
            TypeDefinition::Float => Some(ClassicalType::Float(size)),
            TypeDefinition::Angle => Some(ClassicalType::Angle(size)),
            TypeDefinition::Complex => Some(ClassicalType::Complex(None)),
            TypeDefinition::Duration => Some(ClassicalType::Duration(size)),
            _ => None,
        }
    }

    pub fn get_default_value(&self, size: Option<Expr>) -> Result<Value, RuntimeError> {
        self.get_classical_type(None).unwrap().get_default_value(size)
    }
}