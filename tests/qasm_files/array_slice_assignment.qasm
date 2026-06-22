OPENQASM 3.0;

// Test array slice assignment
output array[int[8], 5] arr;
arr = {1, 2, 3, 4, 5};

// Assign to slice (if supported)
array[int[8], 2] slice = {10, 20};
// arr[1:3] = slice;  // This might not be supported

// For now just test slice access
int[8] a = arr[1];
int[8] b = arr[2];
