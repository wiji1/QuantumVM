OPENQASM 3.0;

// Test qubit slicing
qubit[4] q;
output bit[2] result;

// Apply gates to a slice
h q[1:3];  // Apply H to q[1] and q[2]

result[0] = measure q[1];
result[1] = measure q[2];
