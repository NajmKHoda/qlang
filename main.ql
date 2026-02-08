function main() -> int {
  let nums = [1,2,3,4,5,6,7,8,9,10];
  let filtered = filter(nums, lambda(x: int) { x > 6 });
  let mapped = map(filtered, lambda(x: int) { x + x });
  foreach(mapped, lambda(x: int) { printi(x); });
}

function filter(arr: int[], pred: (int) -> bool) -> int[] {
  let res: int[] = [];
  let i = 0;
  while i < arr.length() {
    let x = arr[i];
    if pred(x) {
      res.append(x);
    }
    i = i + 1;
  }
  return res;
}

function map(arr: int[], fn: (int) -> int) -> int[] {
  let res: int[] = [];
  let i = 0;
  while i < arr.length() {
    let x = arr[i];
    let y = fn(x);
    res.append(y);
    i = i + 1;
  }
  return res;
}

function foreach(arr: int[], fn: (int) -> void) -> void {
  let i = 0;
  while i < arr.length() {
    let x = arr[i];
    fn(x);
    i = i + 1;
  }
}