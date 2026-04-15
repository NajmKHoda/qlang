datasource data1;

table Student from data1 {
  id: int,
  first_name: str,
  last_name: str,
  grade: int,
  teacher_id: int,
  friend_id: int
}

table DoesntExist from data1 {
  foo: int
}

function main() -> int {
  insertStuff();
}

failable function insertStuff() -> void {
  query {
    insert {
      id: 69420,
      first_name: "SHOULDNT",
      last_name: "EXIST",
      grade: 0,
      teacher_id: 20,
      friend_id: 10
    } into Student
  };
}