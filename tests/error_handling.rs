mod common;

use common::*;

#[test]
fn test_undefined_variable() {
    let result = run_test("type_error_undefined_var.qasm");
    assert_type_error(&result);
}

#[test]
fn test_type_mismatch() {
    let result = run_test("type_mismatch.qasm");
    assert_type_error(&result);
}

#[test]
fn test_const_reassignment() {
    let result = run_test("const_reassignment.qasm");
    assert_type_error(&result);
}

#[test]
fn test_gate_arity_mismatch() {
    let result = run_test("gate_arity_mismatch.qasm");
    assert_type_error(&result);
}

#[test]
fn test_missing_return() {
    let result = run_test("missing_return.qasm");
    assert_type_error(&result);
}

#[test]
fn test_gate_on_classical() {
    let result = run_test("gate_on_classical.qasm");
    assert_type_error(&result);
}

#[test]
fn test_measure_classical() {
    let result = run_test("measure_classical.qasm");
    assert_type_error(&result);
}

#[test]
fn test_qubit_arithmetic() {
    let result = run_test("qubit_arithmetic.qasm");
    assert_parse_error(&result);
}

#[test]
fn test_duplicate_param_names() {
    let result = run_test("duplicate_param_names.qasm");
    assert_type_error(&result);
}

#[test]
fn test_undefined_gate() {
    let result = run_test("undefined_gate.qasm");
    assert_type_error(&result);
}

#[test]
fn test_gate_param_mismatch() {
    let result = run_test("gate_param_mismatch.qasm");
    assert_type_error(&result);
}

#[test]
fn test_undeclared_in_scope() {
    let result = run_test("undeclared_in_scope.qasm");
    assert_type_error(&result);
}

#[test]
fn test_undeclared_qubit() {
    let result = run_test("error_undeclared_qubit.qasm");
    assert_type_error(&result);
}

#[test]
fn test_redeclaration() {
    let result = run_test("error_redeclaration.qasm");
    assert_type_error(&result);
}

#[test]
fn test_type_mismatch_array() {
    let result = run_test("error_type_mismatch_array.qasm");
    assert_success(&result);
}

#[test]
fn test_const_modification() {
    let result = run_test("error_const_modification.qasm");
    assert_type_error(&result);
}

#[test]
fn test_return_type_mismatch() {
    let result = run_test("error_return_type_mismatch.qasm");
    assert_type_error(&result);
}

#[test]
fn test_void_assignment() {
    let result = run_test("error_void_assignment.qasm");
    assert_type_error(&result);
}

#[test]
fn test_float_as_index() {
    let result = run_test("error_float_as_index.qasm");
    assert_type_error(&result);
}

#[test]
fn test_gate_wrong_arity() {
    let result = run_test("error_gate_wrong_arity.qasm");
    assert_success(&result);
}

#[test]
fn test_param_wrong_arity() {
    let result = run_test("error_param_wrong_arity.qasm");
    assert_type_error(&result);
}

#[test]
fn test_index_out_of_bounds() {
    let result = run_test("index_out_of_bounds.qasm");
    assert_runtime_error(&result);
}

#[test]
fn test_division_by_zero() {
    let result = run_test("division_by_zero.qasm");
    assert_runtime_error(&result);
}

#[test]
fn test_negative_index() {
    let result = run_test("negative_index.qasm");
    assert_runtime_error(&result);
}

#[test]
fn test_very_large_index() {
    let result = run_test("very_large_index.qasm");
    assert_runtime_error(&result);
}

#[test]
fn test_wrong_version() {
    let result = run_test("wrong_version.qasm");
    assert_parse_error(&result);
}

#[test]
fn test_concat_in_measure() {
    let result = run_test("test_concat_simple.qasm");
    assert_parse_error(&result);
}

#[test]
fn test_array_slice_assignment() {
    let result = run_test("array_slice_assignment.qasm");
    assert_parse_error(&result);
}

#[test]
fn test_empty_array() {
    let result = run_test("edge_empty_array.qasm");
    assert_success(&result);
}

#[test]
fn test_zero_qubits() {
    let result = run_test("edge_zero_qubits.qasm");
    assert_success(&result);
}

#[test]
fn test_popcount_wrong_layer() {
    let result = run_test("popcount.qasm");
    assert_runtime_error(&result);
}

#[test]
fn test_qubit_slicing_wrong_layer() {
    let result = run_test("qubit_slicing.qasm");
    assert_runtime_error(&result);
}

#[test]
fn test_qubit_concat_assignment() {
    let result = run_test("qubit_concat_assignment.qasm");
    assert_runtime_error(&result);
}
