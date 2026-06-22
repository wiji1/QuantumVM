OPENQASM 3.0;

// Test multiple measurements of same qubit
qubit q;
h q;
bit r1 = measure q;
bit r2 = measure q;  // Second measurement should give same result (collapsed)
