table Person {
  int age;
  str name;
}

function main() -> int {
  prints("Name a person:");
  str target_name <- inputs();
  Person[] persons = query {
    select from Person
    where name = target_name
  };
}