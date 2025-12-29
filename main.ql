datasource main_db;

table Person from main_db {
  int age,
  str name
}

function main() -> int {
  Person[] people <- Person [ ];

  prints("Input the name of a person (or STOP):");
  str name <- inputs();
  while name != "STOP" {
    prints("Input their age:");
    int age <- inputi();

    people.append(Person {
      age: age,
      name: name
    });

    prints("Input another person's name (or STOP):");
    name <- inputs();
  }

  query { insert people into Person };

  prints("Remove a person by name (or blank):");
  name <- inputs();
  if name != "" {
    query { delete from Person where name = name };
  }

  prints("Change a person by name (or blank):");
  name <- inputs();
  if name != "" {
    prints("Input new name:");
    str new_name <- inputs();
    prints("Input new age:");
    int new_age <- inputi();
    query {
      update Person set
        name <- new_name,
        age <- new_age
      where name = name
    };
  }
}