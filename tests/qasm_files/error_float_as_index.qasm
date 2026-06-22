OPENQASM 3.0;

// Test using float as array index (should fail at type check)
array[int[8], 5] arr = {1, 2, 3, 4, 5};
float[64] idx = 2.5;
int[8] x = arr[idx];  // Float cannot be array index
