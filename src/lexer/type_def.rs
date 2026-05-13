use crate::interpreter::runtime_error::RuntimeError;
use crate::interpreter::value::Value;
use crate::lexer::{match_keyword, TokenTrait};
use crate::parser::expression::Expr;
use crate::parser::parse_error::ParseError;
use crate::parser::supporting_types::ClassicalType;

#[derive(Debug, Clone)]
pub enum TypeDefinition {
    Qubit, //TODO: Move this to new enum called QuantumTypeDefinition
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
            return Some((Self::Complex, len));
        }
        if let Some(len) = match_keyword(input, "array") {
            return Some((Self::Array, len));
        }
        if let Some(len) = match_keyword(input, "void") {
            return Some((Self::Void, len));
        }
        None
    }
}

impl TypeDefinition {
    pub fn get_classical_type(&self) -> Option<ClassicalType> {
        match self {
            TypeDefinition::Bool => Some(ClassicalType::Bool(None)),
            TypeDefinition::Bit => Some(ClassicalType::Bit(None)),
            TypeDefinition::Int => Some(ClassicalType::Int(None)),
            TypeDefinition::UInt => Some(ClassicalType::UInt(None)),
            TypeDefinition::Float => Some(ClassicalType::Float(None)),
            TypeDefinition::Angle => Some(ClassicalType::Angle(None)),
            TypeDefinition::Complex => Some(ClassicalType::Complex(None)),
            TypeDefinition::Duration => Some(ClassicalType::Duration(None)),
            _ => None,
        }
    }

    pub fn get_default_value(&self, size: Option<Expr>) -> Result<Value, RuntimeError> {
        self.get_classical_type().unwrap().get_default_value(size)
    }
}