OPENQASM 3.0;

// Test measuring classical variable (should fail at type check)
int[32] x = 5;
bit result = measure x;
