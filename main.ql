datasource data;

table Person from data {
  age: int,
  name: str,
  occupation: str
}

function main() -> int {
  let getAllPeople = query() { select from Person };
  let people1 = getAllPeople();
  let people2 = getAllPeople();
  
  while people1.has_next() {
    let person1 = people1.next();
    let person2 = people2.next();
    prints(person1.name);
    prints(person2.name);
  }
}
