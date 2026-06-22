OPENQASM 3.0;

// Test angle type
angle[32] theta = pi / 4;
qubit q;
u(theta, 0, 0) q;
