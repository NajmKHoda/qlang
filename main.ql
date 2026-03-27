datasource data;

table Person from data {
  age: int,
  name: str,
  occupation: str
}

struct CasualPerson {
  name: str,
  age: int
}

function main() -> int {
  let casuals = query {
    select CasualPerson { name: Person.name, age: Person.age }
    from Person
  };

  for casual in casuals {
    prints(casual.name);
    printi(casual.age);
  }
}