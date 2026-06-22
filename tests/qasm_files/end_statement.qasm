OPENQASM 3.0;

// Test end statement (should halt execution)
output int[32] value;
value = 10;

end;

value = 20;  // This should never execute
