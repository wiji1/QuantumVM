OPENQASM 3.0;

// Test RY rotation gate
qubit q;
ry(pi) q;  // π rotation around Y-axis should flip to |1⟩

// StateVector should be [0, 1]
output bit result;
result = measure q;  // Should always be 1
