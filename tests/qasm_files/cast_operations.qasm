OPENQASM 3.0;

// Test explicit type casting
float[64] f = 3.14;
int[32] i = int[32](f);
bool b = bool(1);
