OPENQASM 3.0;

// Test popcount builtin
output uint[8] count1;
output uint[8] count2;
output uint[8] count3;

uint[8] a = 7;   // 0b00000111 - 3 bits set
uint[8] b = 255; // 0b11111111 - 8 bits set
uint[8] c = 0;   // 0b00000000 - 0 bits set

count1 = popcount(a);
count2 = popcount(b);
count3 = popcount(c);
