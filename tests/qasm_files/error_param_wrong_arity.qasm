OPENQASM 3.0;

// Test gate with wrong number of parameters (should fail at type check)
qubit q;
rx(pi/2, pi/4) q;  // RX takes only 1 parameter
