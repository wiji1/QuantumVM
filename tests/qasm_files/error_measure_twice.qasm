OPENQASM 3.0;

// Test measuring the same qubit twice (implementation-dependent behavior)
qubit q;
h q;

bit c1 = measure q;
bit c2 = measure q;  // Second measurement should give same result

output bit[2] result;
result[0] = c1;
result[1] = c2;
