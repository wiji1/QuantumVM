OPENQASM 3.0;

// Test negative array indexing (should fail at runtime)
array[int[32], 5] arr;
arr[0] = 10;
int[32] x = arr[-1];
