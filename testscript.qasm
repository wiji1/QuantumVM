OPENQASM 3.0;
array[int, 4] a = {1, 2, 3, 4};
int total = 0;
for int x in a {
    total = total + x;
}