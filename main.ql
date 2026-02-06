function main() -> int {
  int n <- 1;
  var adder <- lambda (int x) -> int {
    x <- x + n;
    n <- n + 1;
    return x;
  };

  prints("Enter number of iterations");
  int i <- inputi();

  while i > 0 {
    int y <- adder(0);
    printi(y);
    i <- i - 1;
  }

  printi(n);
}