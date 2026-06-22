OPENQASM 3.0;

// Test pow gate modifier
qubit q;
pow(2) @ x q;  // X^2 = identity
