OPENQASM 3.0;

// Test Bell state creation and measurement
qubit[2] q;
output bit[2] result;

h q[0];
cx q[0], q[1];

result[0] = measure q[0];
result[1] = measure q[1];
