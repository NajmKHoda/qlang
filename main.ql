function main() -> int {
  prints("Enter name 1:");
  str name1 <- inputs();
  prints("Enter name 2:");
  str name2 <- inputs();

  if name1 < name2 {
    prints(name1 + " appears in the dictionary first!");
  } else if name1 > name2 {
    prints(name2 + " appears in the dictionary first!");
  } else {
    prints("You entered the same name!");
  }
}