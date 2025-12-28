datasource main_db;

table Person from main_db {
  int age,
  str name
}

function main() -> int {
  prints("Target age:");
  int target_age <- inputi();
  Person[] persons <- query {
    select from Person
    where age = target_age
  };

  int n <- persons.length();
  int i <- 0;
  while i < n {
    prints(persons[i].name);
    i <- i + 1;
  }
}