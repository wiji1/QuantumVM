use crate::interpreter::quantum::gates::*;
use crate::interpreter::quantum::resolved_gate::ResolvedGate;
use crate::interpreter::runtime_error::RuntimeError;
use crate::interpreter::value::Value;
use crate::parser::statement::Stmt;
use crate::parser::supporting_types::{ClassicalType, Param, ParamType};
use std::collections::HashMap;

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

fn builtin_sizeof(args: Vec<Value>) -> Result<Value, RuntimeError> {
    match args.len() {
        1 => match &args[0] {
            Value::Array(arr) => Ok(Value::Int(arr.len() as i64)),
            _ => Err(RuntimeError::TypeMismatch("sizeof requires an array".to_string())),
        },
        2 => {
            let dim = match &args[1] {
                Value::Int(i) => *i as usize,
                _ => return Err(RuntimeError::TypeMismatch("sizeof dimension must be int".to_string())),
            };
            let mut current = &args[0];
            for _ in 0..dim {
                match current {
                    Value::Array(arr) => current = &arr[0],
                    _ => return Err(RuntimeError::TypeMismatch("sizeof dimension out of range".to_string())),
                }
            }
            match current {
                Value::Array(arr) => Ok(Value::Int(arr.len() as i64)),
                _ => Err(RuntimeError::TypeMismatch("sizeof dimension out of range".to_string())),
            }
        }
        _ => Err(RuntimeError::InvalidArgCount(1, args.len())),
    }
}

#[derive(Clone)]
pub enum BuiltInFunction {
    Sin, Cos, Tan, Asin, Acos, Atan,
    Sqrt, Exp, Ln, Floor, Ceil,
    Pow, Mod, Sizeof,
}

impl BuiltInFunction {
    pub fn call(&self, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match self {
            BuiltInFunction::Sin => builtin_sin(args),
            BuiltInFunction::Cos => builtin_cos(args),
            BuiltInFunction::Tan => builtin_tan(args),
            BuiltInFunction::Asin => builtin_asin(args),
            BuiltInFunction::Acos => builtin_acos(args),
            BuiltInFunction::Atan => builtin_atan(args),
            BuiltInFunction::Sqrt => builtin_sqrt(args),
            BuiltInFunction::Exp => builtin_exp(args),
            BuiltInFunction::Ln => builtin_ln(args),
            BuiltInFunction::Floor => builtin_floor(args),
            BuiltInFunction::Ceil => builtin_ceil(args),
            BuiltInFunction::Pow => builtin_pow(args),
            BuiltInFunction::Mod => builtin_mod(args),
            BuiltInFunction::Sizeof => builtin_sizeof(args),
        }
    }

    pub fn get_params(&self) -> Vec<Param> {
        match self {
            BuiltInFunction::Sin | BuiltInFunction::Cos | BuiltInFunction::Tan
            | BuiltInFunction::Asin | BuiltInFunction::Acos | BuiltInFunction::Atan
            | BuiltInFunction::Sqrt | BuiltInFunction::Exp | BuiltInFunction::Ln
            | BuiltInFunction::Floor | BuiltInFunction::Ceil => vec![
                Param { ty: ParamType::Unspecified, name: "x".to_string() }
            ],
            BuiltInFunction::Pow => vec![
                Param { ty: ParamType::Unspecified, name: "base".to_string() },
                Param { ty: ParamType::Unspecified, name: "exponent".to_string() },
            ],
            BuiltInFunction::Mod => vec![
                Param { ty: ParamType::Unspecified, name: "a".to_string() },
                Param { ty: ParamType::Unspecified, name: "b".to_string() },
            ],
            BuiltInFunction::Sizeof => vec![
                Param { ty: ParamType::Unspecified, name: "array".to_string() },
            ],
        }
    }
}

#[derive(Clone)]
pub enum Function {
    BuiltIn(BuiltInFunction),
    BuiltInGate(BuiltInGate),
    UserDefined { params: Vec<Param>, return_type: Option<ClassicalType>, body: Vec<Stmt> },
    Gate { params: Vec<String>, qubits: Vec<String>, body: Vec<Stmt> },
    Extern
}

impl Function {
    pub fn get_params(&self) -> Vec<Param> {
        match self {
            Function::BuiltIn(func) => func.get_params(),
            Function::BuiltInGate(gate) => gate.get_params(),
            Function::UserDefined { params, .. } => params.clone(),
            Function::Gate { params, .. } => {
                params.iter().map(|name| Param {
                    ty: ParamType::Scalar(ClassicalType::Angle(None)),
                    name: name.clone(),
                }).collect()
            },
            Function::Extern => vec![],
        }
    }
}

pub fn get_default_functions() -> HashMap<String, Function> {
    let mut functions: HashMap<String, Function> = HashMap::new();
    functions.insert("sin".to_string(), Function::BuiltIn(BuiltInFunction::Sin));
    functions.insert("cos".to_string(), Function::BuiltIn(BuiltInFunction::Cos));
    functions.insert("tan".to_string(), Function::BuiltIn(BuiltInFunction::Tan));
    functions.insert("arcsin".to_string(), Function::BuiltIn(BuiltInFunction::Asin));
    functions.insert("arccos".to_string(), Function::BuiltIn(BuiltInFunction::Acos));
    functions.insert("arctan".to_string(), Function::BuiltIn(BuiltInFunction::Atan));
    functions.insert("sqrt".to_string(), Function::BuiltIn(BuiltInFunction::Sqrt));
    functions.insert("exp".to_string(), Function::BuiltIn(BuiltInFunction::Exp));
    functions.insert("ln".to_string(), Function::BuiltIn(BuiltInFunction::Ln));
    functions.insert("floor".to_string(), Function::BuiltIn(BuiltInFunction::Floor));
    functions.insert("ceil".to_string(), Function::BuiltIn(BuiltInFunction::Ceil));
    functions.insert("pow".to_string(), Function::BuiltIn(BuiltInFunction::Pow));
    functions.insert("mod".to_string(), Function::BuiltIn(BuiltInFunction::Mod));
    functions.insert("sizeof".to_string(), Function::BuiltIn(BuiltInFunction::Sizeof));

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

    pub fn get_params(&self) -> Vec<Param> {
        match self {
            BuiltInGate::U => vec![
                Param { ty: ParamType::Scalar(ClassicalType::Angle(None)), name: "theta".to_string() },
                Param { ty: ParamType::Scalar(ClassicalType::Angle(None)), name: "phi".to_string() },
                Param { ty: ParamType::Scalar(ClassicalType::Angle(None)), name: "lambda".to_string() },
            ],
            BuiltInGate::Cx => vec![],
        }
    }
}
