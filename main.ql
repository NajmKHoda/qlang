datasource data1;

table Student from data1 {
  id: int,
  first_name: str,
  last_name: str,
  grade: int,
  teacher_id: int,
  friend_id: int
}

function main() -> int {
  transaction {
    let students = query { select all from Student };
    for student in students {
      prints(student.first_name);
    }
  } on rollback {
    prints("Oops! Something went very wrong.");
    return 1;
  }
}