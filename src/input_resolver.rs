use crate::interpreter::value::Value;
use crate::parser::supporting_types::ClassicalType;
use crate::parser::expression::{Expr, ExprKind};
use std::collections::HashMap;
use std::fs;
use std::path::Path;


#[derive(Debug, Clone, PartialEq)]
pub enum InputSource {
    Json,
    Cli,
}

#[derive(Debug, Clone)]
pub enum InputError {
    MissingRequired(String),
    TypeMismatch { name: String, expected: String, got: String, source: InputSource },
    ParseError { name: String, value: String, reason: String },
    JsonFileError(String),
    InvalidArrayDimensions { name: String, expected: Vec<usize>, got: Vec<usize> },
}

impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::MissingRequired(name) =>
                write!(f, "Missing required input variable: {}", name),
            InputError::TypeMismatch { name, expected, got, source } =>
                write!(f, "Type mismatch for input '{}' (from {:?}): expected {}, got {}",
                    name, source, expected, got),
            InputError::ParseError { name, value, reason } =>
                write!(f, "Failed to parse input '{}' with value '{}': {}",
                    name, value, reason),
            InputError::JsonFileError(msg) =>
                write!(f, "JSON file error: {}", msg),
            InputError::InvalidArrayDimensions { name, expected, got } =>
                write!(f, "Invalid array dimensions for '{}': expected {:?}, got {:?}",
                    name, expected, got),
        }
    }
}

pub struct InputResolver {
    json_inputs: HashMap<String, serde_json::Value>,
    cli_inputs: HashMap<String, String>,
}

impl InputResolver {
    pub fn new() -> Self {
        Self {
            json_inputs: HashMap::new(),
            cli_inputs: HashMap::new(),
        }
    }

    pub fn load_json_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), InputError> {
        let path = path.as_ref();
        let path_str = path.display().to_string();

        if !path.exists() {
            return Err(InputError::JsonFileError(
                format!("File not found: '{}'", path_str)
            ));
        }

        if !path.is_file() {
            return Err(InputError::JsonFileError(
                format!("Path is not a file: '{}'", path_str)
            ));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| InputError::JsonFileError(
                format!("Failed to read file '{}': {}", path_str, e)
            ))?;

        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| InputError::JsonFileError(
                format!("Failed to parse JSON from '{}': {}", path_str, e)
            ))?;

        if let serde_json::Value::Object(map) = json {
            self.json_inputs = map.into_iter().collect();
            Ok(())
        } else {
            Err(InputError::JsonFileError(
                format!("JSON root must be an object in '{}', found: {:?}",
                    path_str, json)
            ))
        }
    }

    pub fn add_cli_input(&mut self, key: String, value: String) {
        self.cli_inputs.insert(key, value);
    }

    pub fn resolve(
        &self,
        declarations: &HashMap<String, ClassicalType>,
    ) -> Result<(HashMap<String, Value>, Vec<String>), InputError> {
        let mut resolved = HashMap::new();
        let mut warnings = Vec::new();

        for (name, decl_type) in declarations {
            let value = if self.cli_inputs.contains_key(name) {
                self.parse_and_validate_cli(name, &self.cli_inputs[name], decl_type)?
            } else if self.json_inputs.contains_key(name) {
                self.parse_and_validate_json(name, &self.json_inputs[name], decl_type)?
            } else {
                return Err(InputError::MissingRequired(name.clone()));
            };

            resolved.insert(name.clone(), value);
        }

        for key in self.cli_inputs.keys() {
            if !declarations.contains_key(key) {
                warnings.push(format!("Unused input variable: {}", key));
            }
        }
        for key in self.json_inputs.keys() {
            if !declarations.contains_key(key) && !self.cli_inputs.contains_key(key) {
                warnings.push(format!("Unused input variable: {}", key));
            }
        }

        Ok((resolved, warnings))
    }

    fn parse_and_validate_cli(
        &self,
        name: &str,
        value: &str,
        expected_type: &ClassicalType,
    ) -> Result<Value, InputError> {
        let parsed = parse_cli_value(value)
            .map_err(|reason| InputError::ParseError {
                name: name.to_string(),
                value: value.to_string(),
                reason,
            })?;

        validate_value(name, &parsed, expected_type, InputSource::Cli)
    }

    fn parse_and_validate_json(
        &self,
        name: &str,
        value: &serde_json::Value,
        expected_type: &ClassicalType,
    ) -> Result<Value, InputError> {
        let parsed = json_to_value(value)
            .map_err(|reason| InputError::ParseError {
                name: name.to_string(),
                value: format!("{:?}", value),
                reason,
            })?;

        validate_value(name, &parsed, expected_type, InputSource::Json)
    }
}

fn parse_cli_value(s: &str) -> Result<Value, String> {
    let s = s.trim();

    if s.starts_with('[') && s.ends_with(']') { return parse_array(s); }
    if s == "true" || s == "false" { return Ok(Value::Bool(s == "true")); }

    if s == "0" || s == "1" { return Ok(Value::Int(s.parse().unwrap())); }

    if s.contains('.') || s.contains('e') || s.contains('E') {
        return s.parse::<f64>()
            .map(Value::Float)
            .map_err(|_| format!("Invalid float: {}", s));
    }

    if let Ok(i) = s.parse::<i64>() { return Ok(Value::Int(i)); }

    Err(format!("Cannot parse value: {}", s))
}

fn parse_array(s: &str) -> Result<Value, String> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err("Array must be enclosed in brackets".to_string());
    }

    let inner = &s[1..s.len()-1].trim();
    if inner.is_empty() { return Ok(Value::Array(vec![])); }

    if inner.starts_with('[') {
        let elements = split_array_elements(inner)?;
        let mut result = Vec::new();
        for elem in elements {
            result.push(parse_array(&elem)?);
        }
        Ok(Value::Array(result))
    } else {
        let elements: Result<Vec<Value>, String> = inner
            .split(',')
            .map(|e| parse_cli_value(e.trim()))
            .collect();
        Ok(Value::Array(elements?))
    }
}

fn split_array_elements(s: &str) -> Result<Vec<String>, String> {
    let mut elements = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for ch in s.chars() {
        match ch {
            '[' => {
                depth += 1;
                current.push(ch);
            }
            ']' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                elements.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        elements.push(current.trim().to_string());
    }

    if depth != 0 {
        return Err("Unbalanced brackets in array".to_string());
    }

    Ok(elements)
}

fn json_to_value(json: &serde_json::Value) -> Result<Value, String> {
    match json {
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else { Err("Invalid number".to_string()) }
        }
        serde_json::Value::Array(arr) => {
            let elements: Result<Vec<Value>, String> = arr
                .iter()
                .map(json_to_value)
                .collect();
            Ok(Value::Array(elements?))
        }
        serde_json::Value::String(s) => parse_cli_value(s),
        _ => Err("Unsupported JSON type (must be bool, number, string, or array)".to_string()),
    }
}

fn validate_value(
    name: &str,
    value: &Value,
    expected_type: &ClassicalType,
    source: InputSource,
) -> Result<Value, InputError> {
    match expected_type {
        ClassicalType::Int(width_expr) => {
            let width = get_width_from_expr(width_expr);
            match value {
                Value::Int(i) => Ok(Value::Int(*i)),
                Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
                _ => Err(InputError::TypeMismatch {
                    name: name.to_string(),
                    expected: format!("int[{}]", width.unwrap_or(32)),
                    got: type_name(value),
                    source,
                }),
            }
        }
        ClassicalType::Float(width_expr) => {
            let width = get_width_from_expr(width_expr);
            match value {
                Value::Float(f) => Ok(Value::Float(*f)),
                Value::Int(i) => Ok(Value::Float(*i as f64)),
                _ => Err(InputError::TypeMismatch {
                    name: name.to_string(),
                    expected: format!("float[{}]", width.unwrap_or(64)),
                    got: type_name(value),
                    source,
                }),
            }
        }
        ClassicalType::Bit(width_expr) => {
            let width = get_width_from_expr(width_expr).unwrap_or(1);
            match value {
                Value::Bool(b) => Ok(Value::Bits {
                    value: if *b { 1 } else { 0 },
                    width
                }),
                Value::Int(i) if *i == 0 || *i == 1 => Ok(Value::Bits {
                    value: *i as u64,
                    width
                }),
                Value::Bits { .. } => Ok(value.clone()),
                _ => Err(InputError::TypeMismatch {
                    name: name.to_string(),
                    expected: format!("bit[{}]", width),
                    got: type_name(value),
                    source,
                }),
            }
        }
        ClassicalType::Bool(_) => {
            match value {
                Value::Bool(b) => Ok(Value::Bool(*b)),
                Value::Int(i) if *i == 0 || *i == 1 => Ok(Value::Bool(*i != 0)),
                _ => Err(InputError::TypeMismatch {
                    name: name.to_string(),
                    expected: "bool".to_string(),
                    got: type_name(value),
                    source,
                }),
            }
        }
        ClassicalType::UInt(width_expr) => {
            let width = get_width_from_expr(width_expr);
            match value {
                Value::Int(i) if *i >= 0 => Ok(Value::Int(*i)),
                _ => Err(InputError::TypeMismatch {
                    name: name.to_string(),
                    expected: format!("uint[{}]", width.unwrap_or(32)),
                    got: type_name(value),
                    source,
                }),
            }
        }
        _ => Ok(value.clone())
    }
}

fn get_width_from_expr(expr: &Option<Expr>) -> Option<usize> {
    expr.as_ref().and_then(|e| {
        if let ExprKind::Int(i) = &e.kind {
            Some(*i as usize)
        } else { None }
    })
}

fn type_name(value: &Value) -> String {
    match value {
        Value::Int(_) => "int".to_string(),
        Value::Float(_) => "float".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::Bits { width, .. } => format!("bit[{}]", width),
        Value::Complex(_, _) => "complex".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Qubit(_) => "qubit".to_string(),
        Value::Void => "void".to_string(),
        Value::Range { .. } => "range".to_string(),
        Value::Timing(_) => "duration".to_string(),
    }
}