OPENQASM 3.0;

// Test measure operator in expression
qubit q;
h q;
bit result = measure q;
