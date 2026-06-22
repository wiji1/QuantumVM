OPENQASM 3.0;

// Test using undeclared qubit (should fail at type check)
h q;  // q is not declared
