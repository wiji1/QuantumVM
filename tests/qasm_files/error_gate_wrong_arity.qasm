OPENQASM 3.0;

// Test gate with wrong number of qubits (should fail at type check)
qubit[2] q;
h q[0], q[1];  // H gate takes only 1 qubit
