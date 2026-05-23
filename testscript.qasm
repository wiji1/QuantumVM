OPENQASM 3.0;
int x = 5;
{
    int y = x + 1;
    x = x + 10;
}
y += 1;
int z = x;