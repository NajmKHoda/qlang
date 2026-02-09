datasource data;

table Person from data {
  age: int,
  name: str,
  occupation: str
}

function main() -> int {
  let insert_person = query(_age: int, _name: str, _occupation: str) {
    insert {
      age: _age,
      name: _name,
      occupation: _occupation
    } into Person
  };

  prints("Enter a name (or 'STOP'):");
  let name = inputs();
  while name != "STOP" {
    prints("Age:");
    let age = inputi();
    prints("Occupation:");
    let occupation = inputs();
    insert_person(age, name, occupation);

    prints("Enter another name (or 'STOP'):");
    name = inputs();
  }
}

