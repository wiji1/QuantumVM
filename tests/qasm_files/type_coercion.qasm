OPENQASM 3.0;

// Test type coercion
int[32] i = 5;
float[64] f = i;  // Should coerce int to float
bool b = true;
int[32] i2 = b;   // Should coerce bool to int
