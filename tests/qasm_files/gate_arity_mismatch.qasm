OPENQASM 3.0;

// Test gate arity mismatch (should fail at type check)
qubit[2] q;
cx q[0];  // CX needs 2 qubits
