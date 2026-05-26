OPENQASM 3.0;
gate my_rx(theta) q {
    U(theta, -pi/2, pi/2) q;
}
qubit q;
my_rx(pi) q;