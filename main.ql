table Person {
  str name,
  int age,
  bool is_married
}

function inputp() -> Person {
  prints("Your name:");
  str name <- inputs();
  _print_rc(name);

  prints("Your age:");
  int age <- inputi();

  prints("Are you married (Y/N)?");
  bool is_married <- false;
  if inputs() = "Y" {
    is_married <- true;
  }

  Person person <- Person {
    name: name,
    age: age,
    is_married: is_married
  };
  _print_rc(name);
  
  return person;
}

function main() -> int {
  Person person <- inputp();
  _print_rc(person.name);

  Person person2 <- person;
  _print_rc(person.name);

  prints("Hello, " + person2.name);
}