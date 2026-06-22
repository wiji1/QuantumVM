OPENQASM 3.0;

// Test parameterized gate
gate rz(theta) q {
    u(0, 0, theta) q;
}

qubit q;
rz(pi/4) q;
