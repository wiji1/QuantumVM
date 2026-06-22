OPENQASM 3.0;

// Test very small float values
output float[64] small_val;
output float[64] underflow;

small_val = 1e-308;
underflow = 1e-400;  // Might underflow to 0
