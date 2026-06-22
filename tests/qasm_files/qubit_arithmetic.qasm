OPENQASM 3.0;

// Test arithmetic on qubits (should fail at type check)
qubit q1;
qubit q2;
qubit q3 = q1 + q2;
