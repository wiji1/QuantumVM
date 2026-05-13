OPENQASM 3.0;
array[int, 2, 3] a = {{1, 2, 3}, {4, 5, 6}};
a[0][1] = 99;
int x = a[0][1];