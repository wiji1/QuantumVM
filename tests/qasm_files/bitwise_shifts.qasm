OPENQASM 3.0;

// Test bitwise shift operators
output uint[8] left_shift;
output uint[8] right_shift;
output uint[8] left_shift_overflow;

uint[8] a = 5;  // 0b00000101
left_shift = a << 2;  // Should be 20 (0b00010100)
right_shift = a >> 1;  // Should be 2 (0b00000010)

uint[8] b = 200;  // 0b11001000
left_shift_overflow = b << 1;  // Should be 144 (0b10010000) with overflow
