table Person {
  int age,
  str name,
  bool is_married
}

function main() -> int {
  Person person <- Person {
    name: "Monty Mole",
    age: 67,
    is_married: true
  };
}