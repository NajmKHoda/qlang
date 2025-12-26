table Student {
  int age,
  int score
}

function main() -> int {
  Student[] students <- Student [
    Student { age: 10, score: 100 },
    Student { age: 8, score: 90 },
    Student { age: 7, score: 85 },
    Student { age: 6, score: 60 }
  ];

  printi(students[1].score);
}