OPENQASM 3.0;

// Test gate broadcasting across registers
qubit[4] q;
output bit[4] result;

// Broadcasting: apply H to all qubits
h q;

// All should be in superposition, measure them
result[0] = measure q[0];
result[1] = measure q[1];
result[2] = measure q[2];
result[3] = measure q[3];
