OPENQASM 3.0;
int x = 99;
int result = 0;
switch (x) {
    case 1 { result = 1; }
    default { result = 42; }
}