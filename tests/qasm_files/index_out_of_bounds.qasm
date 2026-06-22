OPENQASM 3.0;

// Test index out of bounds (should fail at runtime, not type check)
qubit[2] q;
h q[5];  // Index out of bounds
