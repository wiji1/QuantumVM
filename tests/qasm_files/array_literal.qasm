OPENQASM 3.0;

// Test array literal initialization
array[int[32], 3] arr = {10, 20, 30};
int[32] x = arr[1];
