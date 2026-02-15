datasource data;

table Person from data {
  age: int,
  name: str,
  occupation: str
}

function get_query_fn() -> () -> Person[] {
  prints("Enter occupation of interest:");
  let _occupation: str = inputs();
  return query() {
    select from Person
    where occupation == _occupation
  };
}

function main() -> int {
  let get_by_age = get_query_fn();
  let people: Person[] = get_by_age();
  
  let i = 0;
  while i < people.length() {
    let person = people[i];
    prints(person.name);
    i = i + 1;
  }
}

