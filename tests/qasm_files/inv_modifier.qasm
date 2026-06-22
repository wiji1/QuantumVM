OPENQASM 3.0;

// Test inverse modifier
qubit q;
output bit result;

// Apply X then its inverse - should return to |0⟩
x q;
inv @ x q;

result = measure q;  // Should be 0
