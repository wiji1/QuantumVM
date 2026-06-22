OPENQASM 3.0;

// Test const reassignment (should fail)
const int[32] N = 10;
N = 20;  // Should error - cannot reassign const
