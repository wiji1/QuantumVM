OPENQASM 3.0;

// Test T gate (π/8 phase gate)
// T gate adds phase π/4 to |1⟩
qubit q;
h q;
t q;

// After H: [1/√2, 1/√2]
// After T: [1/√2, e^(iπ/4)/√2]
