datasource main_db;

table Person from main_db {
  int age,
  str name
}

function main() -> int {
  Person person <- Person {
    age: inputi(),
    name: inputs()
  };

  query { insert person into Person };
}