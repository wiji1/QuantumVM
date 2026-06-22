OPENQASM 3.0;

// Test measuring entire qubit register
qubit[3] q;
h q[0];
h q[1];
h q[2];
output bit[3] result;
result = measure q;
