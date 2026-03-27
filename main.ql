datasource data;

table Student from data {
  id: int,
  first_name: str,
  last_name: str,
  grade: int,
  teacher_id: int
}

table Teacher from data {
  id: int,
  first_name: str,
  last_name: str,
  rating: int
}

struct StudentTeacher {
  student_first: str,
  student_last: str,
  teacher_first: str,
  teacher_last: str,
  grade: int
}

function main() -> int {
  let student_teacher_pairs = query {
    select StudentTeacher {
      student_first: Student.first_name,
      student_last: Student.last_name,
      teacher_first: Teacher.first_name,
      teacher_last: Teacher.last_name,
      grade: Student.grade
    } from Student
    join Teacher on Student.teacher_id == id
  };

  for st in student_teacher_pairs {
    let student_name = st.student_first + " " + st.student_last;
    let teacher_name = st.teacher_first + " " + st.teacher_last;
    prints(student_name + ", " + teacher_name);
  }
}