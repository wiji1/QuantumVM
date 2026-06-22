OPENQASM 3.0;

// Test CNOT gate statevector
qubit[2] q;

// Prepare |11⟩
x q[0];
x q[1];

// Apply CNOT - should flip target since control is 1
cx q[0], q[1];

// StateVector should be [0, 0, 1, 0] representing |10⟩
output bit[2] result;
result[0] = measure q[0];  // Should be 1
result[1] = measure q[1];  // Should be 0
