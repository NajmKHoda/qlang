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
  let foo = query() {
    insert {
      id: 2,
      first_name: "Bird",
      last_name: "Cool",
      grade: 10,
      teacher_id: 2,
      friend_id: 1
    } into Student
  };
}