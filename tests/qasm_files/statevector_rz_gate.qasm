OPENQASM 3.0;

// Test RZ rotation gate
qubit q;
h q;
rz(pi/2) q;  // π/2 rotation around Z-axis

// Should add phase to |1⟩ component
