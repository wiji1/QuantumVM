OPENQASM 3.0;

// Test floating point precision
output float[64] result1;
output float[64] result2;
output bool equal;

result1 = 0.1 + 0.2;
result2 = 0.3;

// Due to floating point precision, these might not be exactly equal
equal = (result1 == result2);
