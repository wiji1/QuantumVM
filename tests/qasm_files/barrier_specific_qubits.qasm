OPENQASM 3.0;

// Test barrier with specific qubits
qubit[3] q;

h q[0];
barrier q[0], q[1];  // Barrier on specific qubits
cx q[0], q[1];
