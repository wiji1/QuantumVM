OPENQASM 3.0;

// Test gate definition with empty body (should be valid)
gate mygate q {
}

qubit q;
mygate q;
