datasource data;

table Person from data {
  age: int,
  name: str,
  occupation: str
}

function main() -> int {
  let getProfessionals = query(occ: str) {
    select from Person
    where occupation == occ
  };
  let teachers = getProfessionals("education");
  let professors = getProfessionals("research");
  
  for professor in professors {
    prints("Professor " + professor.name);
  }

  for teacher in teachers {
    prints("Teacher " + teacher.name);
  }
}