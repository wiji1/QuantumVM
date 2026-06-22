OPENQASM 3.0;

// Test bitwise rotation functions (if supported)
output uint[8] rotated_left;
output uint[8] rotated_right;

uint[8] a = 129;  // 0b10000001
rotated_left = rotl(a, 1);  // Should be 3 (0b00000011)
rotated_right = rotr(a, 1);  // Should be 192 (0b11000000)
