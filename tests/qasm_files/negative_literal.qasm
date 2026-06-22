OPENQASM 3.0;

// Test with explicit negative literal
array[int[32], 5] arr;
arr[0] = 100;
int[32] idx = -1;
int[32] x = arr[idx];
