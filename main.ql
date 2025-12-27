function input_nums() -> int[] {
  int[] nums <- int [ ];
  prints("Integer (or -1 to stop):");
  int num <- inputi();
  while num != -1 {
    nums.append(num);
    prints("Next integer:");
    num <- inputi();
  }

  return nums;
}

function main() -> int {
  int[] favorite_nums <- input_nums();

  while favorite_nums.length() > 0 {
    printi(favorite_nums.pop());
  }
}