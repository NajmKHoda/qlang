function input_wrapper() -> str {
  str x <- inputs();
  return x;
}

function main() -> int {
  str foo <- input_wrapper();
  prints(foo);
}