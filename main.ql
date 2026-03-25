datasource data;

table Person from data {
  age: int,
  name: str,
  occupation: str
}

function main() -> int {
  for i in irange(..) {
    printi(i);
  }
}