OPENQASM 3.0;

// Test Pauli-Y gate statevector
// Y gate should map |0⟩ to i|1⟩
qubit q;
y q;

// StateVector should be [0, i]
output bit result;
result = measure q;  // Should always be 1
