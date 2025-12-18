function sum_range(int a, int b) -> int {
    int s <- 0;
    int j <- a;
    while j <= b {
        s <- s + j;
        j <- j + 1;
    }
    return s;
}

function main() -> int {
    int a <- inputi();
    int b <- inputi();
    int sum <- sum_range(a, b, 10);
    printi(sum);
    return 0;
}