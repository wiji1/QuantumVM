OPENQASM 3.0;

// Test three-qubit statevector
qubit[3] q;

// Create |111⟩
x q[0];
x q[1];
x q[2];

// StateVector should be all zeros except index 7 = 1
// [0, 0, 0, 0, 0, 0, 0, 1]
output bit[3] result;
result = measure q;  // Should be 111 (value 7)
