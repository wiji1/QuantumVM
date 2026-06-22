OPENQASM 3.0;

// Test Pauli-X gate statevector
// X gate should flip |0⟩ to |1⟩
qubit q;
x q;

// StateVector should be [0, 1]
output bit result;
result = measure q;  // Should always be 1
