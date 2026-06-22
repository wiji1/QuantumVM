OPENQASM 3.0;

// Test array operations
array[int[32], 5] arr;
arr[0] = 10;
arr[1] = 20;
int[32] x = arr[0] + arr[1];
