datasource data1;

table Student from data1 {
  id: int,
  first_name: str,
  last_name: str,
  grade: int,
  teacher_id: int,
  friend_id: int
}

failable function main() -> int {
  let i = 0;

  while i < 4 {
    transaction {
      if i == 0 {
        i = i + 1;
        continue;
      }

      if i == 1 {
        i = i + 1;
        break;
      }

      i = i + 1;
    } on rollback {
      printi(-100);
    }
  }

  transaction {
    return 42;
  } on rollback {
    printi(-200);
  }
}