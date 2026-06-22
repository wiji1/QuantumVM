OPENQASM 3.0;

// Test that type checker catches undefined variable (should fail at type check)
bit result = measure q[0];
