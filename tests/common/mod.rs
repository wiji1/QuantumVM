use quantum_vm::{run_file, ExecutionResult, Value};

pub fn run_test(filename: &str) -> ExecutionResult {
    let path = format!("tests/qasm_files/{}", filename);
    run_file(&path).expect("Failed to read test file")
}

pub fn assert_success(result: &ExecutionResult) {
    match result {
        ExecutionResult::Success { .. } => {},
        ExecutionResult::ParseError(e) => panic!("Expected success but got parse error: {:?}", e),
        ExecutionResult::TypeCheckError(e) => panic!("Expected success but got type check error: {:?}", e),
        ExecutionResult::RuntimeError(e) => panic!("Expected success but got runtime error: {}", e),
    }
}

pub fn assert_type_error(result: &ExecutionResult) {
    match result {
        ExecutionResult::TypeCheckError(_) => {},
        ExecutionResult::Success { .. } => panic!("Expected type check error but succeeded"),
        ExecutionResult::ParseError(e) => panic!("Expected type check error but got parse error: {:?}", e),
        ExecutionResult::RuntimeError(e) => panic!("Expected type check error but got runtime error: {}", e),
    }
}

pub fn assert_runtime_error(result: &ExecutionResult) {
    match result {
        ExecutionResult::RuntimeError(_) => {},
        ExecutionResult::Success { .. } => panic!("Expected runtime error but succeeded"),
        ExecutionResult::ParseError(e) => panic!("Expected runtime error but got parse error: {:?}", e),
        ExecutionResult::TypeCheckError(e) => panic!("Expected runtime error but got type check error: {:?}", e),
    }
}

pub fn assert_parse_error(result: &ExecutionResult) {
    match result {
        ExecutionResult::ParseError(_) => {},
        ExecutionResult::Success { .. } => panic!("Expected parse error but succeeded"),
        ExecutionResult::TypeCheckError(e) => panic!("Expected parse error but got type check error: {:?}", e),
        ExecutionResult::RuntimeError(e) => panic!("Expected parse error but got runtime error: {}", e),
    }
}

pub fn get_output<'a>(result: &'a ExecutionResult, name: &str) -> &'a Value {
    result.get_output(name)
        .unwrap_or_else(|| panic!("Output '{}' not found", name))
}

pub fn assert_int_output(result: &ExecutionResult, name: &str, expected: i64) {
    let value = get_output(result, name);
    match value {
        Value::Int(actual) => {
            assert_eq!(*actual, expected, "Output '{}' mismatch", name);
        }
        _ => panic!("Output '{}' is not an integer: {:?}", name, value),
    }
}

pub fn assert_float_output(result: &ExecutionResult, name: &str, expected: f64, tolerance: f64) {
    let value = get_output(result, name);
    match value {
        Value::Float(actual) => {
            let diff = (actual - expected).abs();
            assert!(
                diff < tolerance,
                "Output '{}' mismatch: expected {}, got {} (diff: {})",
                name, expected, actual, diff
            );
        }
        _ => panic!("Output '{}' is not a float: {:?}", name, value),
    }
}

pub fn assert_bool_output(result: &ExecutionResult, name: &str, expected: bool) {
    let value = get_output(result, name);
    match value {
        Value::Bool(actual) => {
            assert_eq!(*actual, expected, "Output '{}' mismatch", name);
        }
        _ => panic!("Output '{}' is not a boolean: {:?}", name, value),
    }
}

pub fn assert_bits_output(result: &ExecutionResult, name: &str, expected_value: u64) {
    let value = get_output(result, name);
    match value {
        Value::Bits { value, .. } => {
            assert_eq!(*value, expected_value, "Output '{}' mismatch", name);
        }
        _ => panic!("Output '{}' is not bits: {:?}", name, value),
    }
}
