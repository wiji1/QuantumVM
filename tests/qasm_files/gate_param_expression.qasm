OPENQASM 3.0;

// Test gate parameter expressions
gate myrot(theta, phi) q {
    rz(theta + phi) q;
    rx(theta * 2) q;
}

qubit q;
myrot(pi/4, pi/8) q;
