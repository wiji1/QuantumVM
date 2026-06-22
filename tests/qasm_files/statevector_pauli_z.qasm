OPENQASM 3.0;

// Test Pauli-Z gate statevector
// Z gate should leave |0⟩ unchanged
qubit q;
z q;

// StateVector should be [1, 0]
output bit result;
result = measure q;  // Should always be 0
