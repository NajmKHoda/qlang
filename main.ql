function main() -> int {
  prints("Enter your first word:");
  str total <- inputs();

  "useless_string";

  while true {
    prints("Enter the next word, or STOP to conclude:");
    str segment <- inputs();
    if segment = "STOP" {
      break;
    }
    total <- total + " " + segment;
  }

  prints("Your sentence is: " + total);
}