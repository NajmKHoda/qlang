n = inputi();

a = 0;
b = 1;
while n > 0 {
    printi(b);
    b = b + a;
    a = b - a;
    n = n - 1;
}
