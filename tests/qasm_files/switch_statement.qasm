OPENQASM 3.0;

// Test switch statement
int[32] x = 2;
int[32] result;
switch (x) {
    case 1 { result = 10; }
    case 2 { result = 20; }
    default { result = 0; }
}
