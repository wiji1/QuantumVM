OPENQASM 3.0;

// Test quantum if (conditional based on measurement)
qubit q;
h q;
bit c = measure q;
if (c) {
    // Do something if measured 1
    int[32] x = 1;
} else {
    // Do something if measured 0
    int[32] x = 0;
}
