datasource data;

table Student from data {
  id: int,
  first_name: str,
  last_name: str,
  grade: int,
  teacher_id: int,
  friend_id: int
}

struct NameOnly {
  first: str
}

function main() -> int {
  let no_rows = query(max_rows: int) {
    select NameOnly {
      first: S.first_name
    } from Student as S
    limit max_rows
  };

  for row in no_rows(-3) {
    prints(row.first);
  }

  let one_row = query(rows_to_skip: int) {
    select NameOnly {
      first: S.first_name
    } from Student as S
    limit 1
    offset rows_to_skip
  };

  for row in one_row(1) {
    prints(row.first);
  }

  return 0;
}