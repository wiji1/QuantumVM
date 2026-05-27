use std::collections::HashMap;
use crate::interpreter::quantum::gates::*;
use crate::interpreter::quantum::resolved_gate::ResolvedGate;
use crate::interpreter::quantum::statevector::StateVector;
use crate::interpreter::runtime_error::RuntimeError;
use crate::interpreter::value::Value;
use crate::lexer::type_def::TypeDefinition;
use crate::parser::expression::Expr;
use crate::parser::statement::Stmt;
use crate::parser::supporting_types::Param;

macro_rules! math_builtin {
    ($name:ident, $method:ident) => {
        fn $name(args: Vec<Value>) -> Result<Value, RuntimeError> {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidArgCount(1, args.len()));
            }
            match args[0] {
                Value::Float(f) => Ok(Value::Float(f.$method())),
                Value::Int(i) => Ok(Value::Float((i as f64).$method())),
                _ => Err(RuntimeError::TypeMismatch(
                    format!("{} requires numeric argument", stringify!($name))
                )),
            }
        }
    };
}

math_builtin!(builtin_sin, sin);
math_builtin!(builtin_cos, cos);
math_builtin!(builtin_tan, tan);
math_builtin!(builtin_asin, asin);
math_builtin!(builtin_acos, acos);
math_builtin!(builtin_atan, atan);
math_builtin!(builtin_sqrt, sqrt);
math_builtin!(builtin_exp, exp);
math_builtin!(builtin_ln, ln);
math_builtin!(builtin_floor, floor);
math_builtin!(builtin_ceil, ceil);

fn builtin_pow(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount(2, args.len()));
    }
    let base = match &args[0] {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => return Err(RuntimeError::TypeMismatch("pow requires numeric arguments".to_string())),
    };
    let exp = match &args[1] {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => return Err(RuntimeError::TypeMismatch("pow requires numeric arguments".to_string())),
    };
    Ok(Value::Float(base.powf(exp)))
}

fn builtin_mod(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount(2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 % b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a % *b as f64)),
        _ => Err(RuntimeError::TypeMismatch("mod requires numeric arguments".to_string())),
    }
}

#[derive(Clone)]
pub enum Function {
    BuiltIn(fn(Vec<Value>) -> Result<Value, RuntimeError>),
    BuiltInGate(BuiltInGate),
    UserDefined { params: Vec<Param>, return_type: Option<(TypeDefinition, Option<Expr>)>, body: Vec<Stmt> },
    Gate { params: Vec<String>, qubits: Vec<String>, body: Vec<Stmt> },
}

pub fn get_default_functions() -> HashMap<String, Function> {
    let mut functions: HashMap<String, Function> = HashMap::new();
    functions.insert("sin".to_string(), Function::BuiltIn(builtin_sin));
    functions.insert("cos".to_string(), Function::BuiltIn(builtin_cos));
    functions.insert("tan".to_string(), Function::BuiltIn(builtin_tan));
    functions.insert("asin".to_string(), Function::BuiltIn(builtin_asin));
    functions.insert("acos".to_string(), Function::BuiltIn(builtin_acos));
    functions.insert("atan".to_string(), Function::BuiltIn(builtin_atan));
    functions.insert("sqrt".to_string(), Function::BuiltIn(builtin_sqrt));
    functions.insert("exp".to_string(), Function::BuiltIn(builtin_exp));
    functions.insert("ln".to_string(), Function::BuiltIn(builtin_ln));
    functions.insert("floor".to_string(), Function::BuiltIn(builtin_floor));
    functions.insert("ceil".to_string(), Function::BuiltIn(builtin_ceil));
    functions.insert("pow".to_string(), Function::BuiltIn(builtin_pow));
    functions.insert("mod".to_string(), Function::BuiltIn(builtin_mod));

    functions.insert("U".to_string(),  Function::BuiltInGate(BuiltInGate::U));
    functions.insert("CX".to_string(), Function::BuiltInGate(BuiltInGate::Cx));

    functions
}

#[derive(Clone)]
pub enum BuiltInGate {
    U,
    Cx
}

impl BuiltInGate {
    pub fn get_resolved(&self, params: &[f64]) -> Result<ResolvedGate, RuntimeError> {
        match self {
            BuiltInGate::U  => Ok(ResolvedGate::SingleQubit(gate_u(params[0], params[1], params[2]))),
            BuiltInGate::Cx => Ok(ResolvedGate::TwoQubit(gate_cx())),
        }
    }
}
