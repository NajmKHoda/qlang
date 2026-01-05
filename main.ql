readonly datasource example;

readonly table Person from example {
  int age,
  str name
}

function main() -> int {
  Person entry <- {
    age: 20,
    name: "Wallace"
  };
  query { insert entry into Person };
}