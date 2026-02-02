datasource example;

table Person from example {
  int age,
  str name
}

function main() -> int {
  prints("Enter a person's age and name:");
  Person entry <- {
    age: inputi(),
    name: inputs()
  };

  if entry.name = "Jerry" {
    prints("Nevermind I HATE Jerry.");
  } else {
    query { insert entry into Person };
  }
}