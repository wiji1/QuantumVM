OPENQASM 3.0;

// Test variable redeclaration (should fail at type check)
int[32] x = 5;
int[32] x = 10;  // Redeclaring x
