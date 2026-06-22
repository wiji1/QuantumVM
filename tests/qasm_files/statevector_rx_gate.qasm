OPENQASM 3.0;

// Test RX rotation gate
qubit q;
rx(pi) q;  // π rotation around X-axis should equal X gate

// StateVector should be [0, -i] (up to global phase)
output bit result;
result = measure q;  // Should always be 1
