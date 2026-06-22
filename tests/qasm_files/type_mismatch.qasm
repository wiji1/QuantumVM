OPENQASM 3.0;

// Test type mismatch detection (should fail at type check)
qubit q;
int[32] x = q;  // Cannot assign qubit to int
