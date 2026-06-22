OPENQASM 3.0;

// Test array type mismatch (should fail at type check)
array[int[8], 3] arr = {1, 2, 3};
float[64] x = arr[0];  // Type mismatch: int[8] to float[64]
