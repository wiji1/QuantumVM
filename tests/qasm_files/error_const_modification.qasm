OPENQASM 3.0;

// Test modifying const (should fail at type check)
const int[32] x = 5;
x = 10;  // Cannot modify const
