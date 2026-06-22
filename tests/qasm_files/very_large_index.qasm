OPENQASM 3.0;

// Test very large index (should fail at runtime)
array[int[32], 5] arr;
int[32] x = arr[1000];
