function main() -> int {
  var nums <- [1,2,3,4,5,6,7,8,9,10];
  var filtered <- filter(nums, lambda(int x) { x > 6 });
  var mapped <- map(filtered, lambda(int x) { x + x });
  foreach(mapped, lambda(int x) { printi(x); });
}

function filter(int[] arr, (int) -> bool pred) -> int[] {
  int[] res <- [];
  int i <- 0;
  while i < arr.length() {
    int x <- arr[i];
    if pred(x) {
      res.append(x);
    }
    i <- i + 1;
  }
  return res;
}

function map(int[] arr, (int) -> int fn) -> int[] {
  int[] res <- [];
  int i <- 0;
  while i < arr.length() {
    int x <- arr[i];
    int y <- fn(x);
    res.append(y);
    i <- i + 1;
  }
  return res;
}

function foreach(int[] arr, (int) -> void fn) -> void {
  int i <- 0;
  while i < arr.length() {
    int x <- arr[i];
    fn(x);
    i <- i + 1;
  }
}