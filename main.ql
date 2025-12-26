function main() -> int {
  str[][] matrix <- str[] [
    str [ "a", "b", "c", "d", "e" ],
    str [ "f", "g", "h", "i", "j" ]
  ];

  str concat <- "";
  int i <- 0;
  while i < 2 {
    int j <- 0;
    while j < 5 {
      concat <- concat + matrix[i][j];
      j <- j + 1;
    }
    i <- i + 1;
  }

  prints(concat);
}