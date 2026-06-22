OPENQASM 3.0;

// Test Hadamard gate statevector
// Single qubit Hadamard should create equal superposition
qubit q;
h q;

// StateVector should be [1/√2, 1/√2]
// When measured, should be 50/50 |0⟩ or |1⟩
