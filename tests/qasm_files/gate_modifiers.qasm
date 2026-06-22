OPENQASM 3.0;

// Test gate modifiers (inv, ctrl)
qubit[2] q;
h q[0];
inv @ x q[0];      // Inverse X (which is X itself)
ctrl @ x q[0], q[1];  // Controlled-X (CNOT)
