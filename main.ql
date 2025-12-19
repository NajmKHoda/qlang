function main() -> int {
    int x <- 1;
    while true {
        while true {
            if x > 10 {
                break Inner;
            } else {
                printi(x);
                x <- x + 1;
            }
        } as Inner;
    } as Outer;
}