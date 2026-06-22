OPENQASM 3.0;

// Test gate with wrong number of parameters (should fail at type check)
qubit q;
rx(pi/4, pi/2) q;  // rx takes 1 parameter, not 2
