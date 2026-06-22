OPENQASM 3.0;

// Test basic qubit declaration and simple gates
qubit[2] q;
h q[0];
cx q[0], q[1];
bit[2] result;
result[0] = measure q[0];
result[1] = measure q[1];
