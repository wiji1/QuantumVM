OPENQASM 3.0;

// Test S gate (phase gate)
// S gate adds phase π/2 to |1⟩
qubit q;
h q;
s q;

// After H: [1/√2, 1/√2]
// After S: [1/√2, i/√2]
