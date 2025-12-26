table Person {
  str name,
  int age,
  bool is_married
}

function inputp() -> Person {
  prints("Your name:");
  str name <- inputs();

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
  
  return person;
}

function main() -> int {
  Person person <- inputp();
  prints("Hello, " + person.name);
}