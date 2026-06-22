mod common;

use common::*;

#[test]
fn test_if_statement() {
    let result = run_test("control_flow_if.qasm");
    assert_success(&result);
}

#[test]
fn test_for_loop() {
    let result = run_test("control_flow_for.qasm");
    assert_success(&result);
}

#[test]
fn test_while_loop() {
    let result = run_test("while_loop.qasm");
    assert_success(&result);
}

#[test]
fn test_switch_statement() {
    let result = run_test("switch_statement.qasm");
    assert_success(&result);
}

#[test]
fn test_break_continue() {
    let result = run_test("break_continue.qasm");
    assert_success(&result);
}

#[test]
fn test_range_step() {
    let result = run_test("range_step.qasm");
    assert_success(&result);
}

#[test]
fn test_quantum_if() {
    let result = run_test("quantum_if.qasm");
    assert_success(&result);
}

#[test]
fn test_nested_loops() {
    let result = run_test("nested_loops.qasm");
    assert_success(&result);
    assert_int_output(&result, "sum", 12);
}

#[test]
fn test_break_in_nested() {
    let result = run_test("break_in_nested.qasm");
    assert_success(&result);
    assert_int_output(&result, "count", 18);
}

#[test]
#[should_panic]
fn test_range_negative_step() {
    let result = run_test("range_negative_step.qasm");
    assert_success(&result);
    assert_int_output(&result, "sum", 30);
}

#[test]
fn test_continue_in_while() {
    let result = run_test("continue_in_while.qasm");
    assert_success(&result);
    assert_int_output(&result, "sum", 25);
}

#[test]
#[should_panic]
fn test_end_statement() {
    let result = run_test("end_statement.qasm");
    assert_success(&result);
    assert_int_output(&result, "value", 10);
}

#[test]
fn test_infinite_loop_with_break() {
    let result = run_test("error_infinite_loop.qasm");
    assert_success(&result);
    assert_int_output(&result, "result", 101);
}

#[test]
fn test_shadowing() {
    let result = run_test("shadowing.qasm");
    assert_success(&result);
}

#[test]
fn test_scope() {
    let result = run_test("scope_test.qasm");
    assert_success(&result);
}
