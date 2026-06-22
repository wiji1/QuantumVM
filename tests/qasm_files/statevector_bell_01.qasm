OPENQASM 3.0;

// Test Bell state |Φ-⟩ = (|00⟩ - |11⟩)/√2
qubit[2] q;

h q[0];
z q[0];
cx q[0], q[1];

// StateVector should be [1/√2, 0, 0, -1/√2]
