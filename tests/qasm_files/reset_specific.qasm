OPENQASM 3.0;

// Test reset on specific qubits
qubit[2] q;
output bit[2] result;

// Set both to |1⟩
x q[0];
x q[1];

// Reset only first qubit
reset q[0];

result[0] = measure q[0];  // Should be 0
result[1] = measure q[1];  // Should be 1
