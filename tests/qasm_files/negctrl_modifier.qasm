OPENQASM 3.0;

// Test negctrl gate modifier (control on |0>)
qubit[2] q;
negctrl @ x q[0], q[1];
