function foo1(int a) -> (int) -> (int) -> (int) -> int {
  return lambda (int b) -> (int) -> (int) -> int {
    return lambda (int c) -> (int) -> int {
      return lambda (int d) -> int {
        return a + b + c + d;
      };
    };
  };
}

function main() -> int {
  var foo2 <- foo1(5);
  var foo3 <- foo2(10);
  var foo4 <- foo3(15);
  var foo5 <- foo4(20);
  printi(foo5);
}