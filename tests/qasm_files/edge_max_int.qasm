OPENQASM 3.0;

// Test maximum integer values
output int[32] max_val;
output int[32] overflow;

max_val = 2147483647;  // Max int32
overflow = max_val + 1;  // Should overflow to negative
