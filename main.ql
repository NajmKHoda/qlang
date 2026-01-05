datasource example;

table Person from example {
  int age,
  str name
}

function main() -> int {
  var foo <- Person { name: "foo" };
}