mod common;

use common::*;

#[test]
fn test_basic_qubit_ops() {
    let result = run_test("basic_qubit_ops.qasm");
    assert_success(&result);
}

#[test]
fn test_bell_state() {
    let result = run_test("bell_state.qasm");
    assert_success(&result);
    let bits = get_output(&result, "result");
    match bits {
        quantum_vm::Value::Bits { value, .. } => {
            assert!(*value == 0 || *value == 3);
        }
        _ => panic!("Expected bits output"),
    }
}

#[test]
fn test_hadamard() {
    let result = run_test("statevector_hadamard.qasm");
    assert_success(&result);
}

#[test]
fn test_pauli_x() {
    let result = run_test("statevector_pauli_x.qasm");
    assert_success(&result);
    assert_bits_output(&result, "result", 1);
}

#[test]
fn test_pauli_y() {
    let result = run_test("statevector_pauli_y.qasm");
    assert_success(&result);
    assert_bits_output(&result, "result", 1);
}

#[test]
fn test_pauli_z() {
    let result = run_test("statevector_pauli_z.qasm");
    assert_success(&result);
    assert_bits_output(&result, "result", 0);
}

#[test]
fn test_s_gate() {
    let result = run_test("statevector_s_gate.qasm");
    assert_success(&result);
}

#[test]
fn test_t_gate() {
    let result = run_test("statevector_t_gate.qasm");
    assert_success(&result);
}

#[test]
fn test_rx_gate() {
    let result = run_test("statevector_rx_gate.qasm");
    assert_success(&result);
    assert_bits_output(&result, "result", 1);
}

#[test]
fn test_ry_gate() {
    let result = run_test("statevector_ry_gate.qasm");
    assert_success(&result);
    assert_bits_output(&result, "result", 1);
}

#[test]
fn test_rz_gate() {
    let result = run_test("statevector_rz_gate.qasm");
    assert_success(&result);
}

#[test]
fn test_cnot() {
    let result = run_test("statevector_cnot.qasm");
    assert_success(&result);
    assert_bits_output(&result, "result", 1);
}

#[test]
fn test_bell_state_phi_plus() {
    let result = run_test("statevector_bell_00.qasm");
    assert_success(&result);
}

#[test]
fn test_bell_state_phi_minus() {
    let result = run_test("statevector_bell_01.qasm");
    assert_success(&result);
}

#[test]
fn test_three_qubit_state() {
    let result = run_test("statevector_three_qubit.qasm");
    assert_success(&result);
    assert_bits_output(&result, "result", 7);
}

#[test]
fn test_multi_ctrl_gate() {
    let result = run_test("multi_ctrl_gate.qasm");
    assert_success(&result);
    assert_bits_output(&result, "result", 7);
}

#[test]
fn test_inv_modifier() {
    let result = run_test("inv_modifier.qasm");
    assert_success(&result);
    assert_bits_output(&result, "result", 0);
}

#[test]
fn test_gate_broadcasting() {
    let result = run_test("gate_broadcasting.qasm");
    assert_success(&result);
}

#[test]
#[should_panic]
fn test_qubit_slicing() {
    let result = run_test("qubit_slicing.qasm");
    assert_success(&result);
}

#[test]
fn test_barrier() {
    let result = run_test("barrier_specific_qubits.qasm");
    assert_success(&result);
}

#[test]
#[should_panic]
fn test_reset() {
    let result = run_test("reset_specific.qasm");
    assert_success(&result);
    assert_bits_output(&result, "result", 1);
}

#[test]
fn test_measure_register() {
    let result = run_test("quantum_register_measure_all.qasm");
    assert_success(&result);
}

#[test]
fn test_measure_twice() {
    let result = run_test("error_measure_twice.qasm");
    assert_success(&result);
}

#[test]
fn test_gate_definition() {
    let result = run_test("gate_definition.qasm");
    assert_success(&result);
}

#[test]
fn test_parameterized_gate() {
    let result = run_test("parameterized_gate.qasm");
    assert_type_error(&result);
}

#[test]
fn test_gate_modifiers() {
    let result = run_test("gate_modifiers.qasm");
    assert_success(&result);
}

#[test]
fn test_pow_modifier() {
    let result = run_test("pow_modifier.qasm");
    assert_success(&result);
}

#[test]
fn test_negctrl_modifier() {
    let result = run_test("negctrl_modifier.qasm");
    assert_success(&result);
}

#[test]
fn test_capital_u_gate() {
    let result = run_test("capital_U_gate.qasm");
    assert_success(&result);
}

#[test]
fn test_gphase() {
    let result = run_test("gphase.qasm");
    assert_success(&result);
}

#[test]
fn test_empty_gate_body() {
    let result = run_test("gate_def_no_body.qasm");
    assert_success(&result);
}

#[test]
fn test_nested_gate_calls() {
    let result = run_test("nested_gate_calls.qasm");
    assert_success(&result);
    let bits = get_output(&result, "result");
    match bits {
        quantum_vm::Value::Bits { value, .. } => {
            assert!(*value == 0 || *value == 3);
        }
        _ => panic!("Expected bits output"),
    }
}

#[test]
#[should_panic]
fn test_gate_param_expression() {
    let result = run_test("gate_param_expression.qasm");
    assert_success(&result);
}

#[test]
fn test_include_stdgates() {
    let result = run_test("include_stdgates.qasm");
    assert_success(&result);
}

#[test]
fn test_measure_operator() {
    let result = run_test("measure_operator.qasm");
    assert_success(&result);
}

#[test]
fn test_multiple_measurements() {
    let result = run_test("multiple_measurements.qasm");
    assert_success(&result);
}

#[test]
fn test_qubit_register_measure() {
    let result = run_test("qubit_register_measure.qasm");
    assert_success(&result);
}
