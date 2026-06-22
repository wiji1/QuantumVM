OPENQASM 3.0;

// Test multiple control qubits (Toffoli gate)
qubit[3] q;
output bit[3] result;

// Prepare control qubits in |11⟩
x q[0];
x q[1];

// Apply Toffoli: should flip q[2] since both controls are 1
ctrl(2) @ x q[0], q[1], q[2];

result[0] = measure q[0];
result[1] = measure q[1];
result[2] = measure q[2];
