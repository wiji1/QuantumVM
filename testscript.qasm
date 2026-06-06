OPENQASM 3.0;
qubit q;

// Global variable
float[64] theta = 1.57;

// VALID: The parameter 'theta' shadows the global 'theta' inside this block
gate my_rx(theta) target {
    // The interpreter must use the passed parameter, not the global 1.57
    U(theta, 0, 0) target;
}

// Global 'theta' remains unaffected here
my_rx(3.14) q;