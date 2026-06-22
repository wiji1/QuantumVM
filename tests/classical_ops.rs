mod common;

use common::*;

#[test]
fn test_classical_ops() {
    let result = run_test("classical_ops.qasm");
    assert_success(&result);
}

#[test]
fn test_bitwise_shifts() {
    let result = run_test("bitwise_shifts.qasm");
    assert_success(&result);
    assert_int_output(&result, "left_shift", 20);
    assert_int_output(&result, "right_shift", 2);
}

#[test]
#[should_panic]
fn test_bitwise_rotation() {
    let result = run_test("bitwise_rotation.qasm");
    assert_success(&result);
}

#[test]
#[should_panic]
fn test_popcount() {
    let result = run_test("popcount.qasm");
    assert_success(&result);
}

#[test]
fn test_bit_operations() {
    let result = run_test("bit_operations.qasm");
    assert_success(&result);
}

#[test]
fn test_complex_numbers() {
    let result = run_test("complex_numbers.qasm");
    assert_success(&result);
}

#[test]
fn test_builtin_functions() {
    let result = run_test("builtin_functions.qasm");
    assert_success(&result);
}

#[test]
fn test_array_operations() {
    let result = run_test("array_operations.qasm");
    assert_success(&result);
}

#[test]
fn test_array_literal() {
    let result = run_test("array_literal.qasm");
    assert_success(&result);
}

#[test]
fn test_cast_operations() {
    let result = run_test("cast_operations.qasm");
    assert_success(&result);
}

#[test]
fn test_type_coercion() {
    let result = run_test("type_coercion.qasm");
    assert_success(&result);
}

#[test]
fn test_const_declaration() {
    let result = run_test("const_declaration.qasm");
    assert_success(&result);
}

#[test]
fn test_let_statement() {
    let result = run_test("let_statement.qasm");
    assert_success(&result);
}

#[test]
fn test_modulo_negative() {
    let result = run_test("modulo_negative.qasm");
    assert_success(&result);
    assert_int_output(&result, "result1", -1);
    assert_int_output(&result, "result2", -2);
}

#[test]
#[should_panic]
fn test_power_operator() {
    let result = run_test("power_operator.qasm");
    assert_success(&result);
    assert_int_output(&result, "result1", 1024);
    assert_float_output(&result, "result2", 1.414, 0.001);
    assert_int_output(&result, "result3", 27);
}

#[test]
fn test_comparison_chain() {
    let result = run_test("comparison_chain.qasm");
    assert_success(&result);
    assert_bool_output(&result, "result1", true);
    assert_bool_output(&result, "result2", true);
}

#[test]
fn test_unary_minus() {
    let result = run_test("unary_minus.qasm");
    assert_success(&result);
    assert_int_output(&result, "result1", -5);
    assert_float_output(&result, "result2", -3.14, 0.001);
}

#[test]
#[should_panic]
fn test_compound_assignment() {
    let result = run_test("compound_assignment.qasm");
    assert_success(&result);
    assert_int_output(&result, "result1", 15);
    assert_int_output(&result, "result2", 12);
    assert_int_output(&result, "result3", 24);
    assert_int_output(&result, "result4", 6);
    assert_int_output(&result, "result5", 2);
}

#[test]
fn test_boolean_ops() {
    let result = run_test("boolean_ops.qasm");
    assert_success(&result);
}

#[test]
fn test_negative_literal() {
    let result = run_test("negative_literal.qasm");
    assert_runtime_error(&result);
}

#[test]
fn test_scientific_notation() {
    let result = run_test("scientific_notation.qasm");
    assert_success(&result);
}

#[test]
fn test_string_ops() {
    let result = run_test("string_ops.qasm");
    assert_success(&result);
}

#[test]
fn test_float_precision() {
    let result = run_test("edge_float_precision.qasm");
    assert_success(&result);
    assert_bool_output(&result, "equal", false);
}

#[test]
fn test_very_small_float() {
    let result = run_test("edge_very_small_float.qasm");
    assert_success(&result);
}

#[test]
#[should_panic]
fn test_max_int_overflow() {
    let result = run_test("edge_max_int.qasm");
    assert_success(&result);
    assert_int_output(&result, "max_val", 2147483647);
    assert_int_output(&result, "overflow", -2147483648);
}

#[test]
fn test_angle_type() {
    let result = run_test("angle_type.qasm");
    assert_runtime_error(&result);
}
