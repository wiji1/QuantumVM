OPENQASM 3.0;

// Test custom gate definition
gate mygate q {
    h q;
    x q;
}

qubit q;
mygate q;
