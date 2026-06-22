OPENQASM 3.0;

// Test measuring entire quantum register
qubit[3] q;
output bit[3] result;

h q[0];
cx q[0], q[1];
cx q[1], q[2];

// Measure all qubits at once
result = measure q;
