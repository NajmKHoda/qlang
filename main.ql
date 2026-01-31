datasource example;

table Person from example {
  int age,
  str name
}

function main() -> int {
  Person entry <- {
    age: inputi(),
    name: inputs()
  };
  query { insert entry into Person };
}