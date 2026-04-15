datasource data1;

table Student from data1 {
  id: int,
  first_name: str,
  last_name: str,
  grade: int,
  teacher_id: int,
  friend_id: int
}

struct FriendPair {
  friend1_first: str,
  friend1_last: str,
  friend2_first: str,
  friend2_last: str
}

function main() -> int {
  let friend_pairs = query {
    select FriendPair {
      friend1_first: Student1.first_name,
      friend1_last: Student1.last_name,
      friend2_first: Student2.first_name,
      friend2_last: Student2.last_name
    } from Student as Student1
    join Student as Student2 on Student1.friend_id == id
  };

  for pair in friend_pairs {
    let friend1_name = pair.friend1_first + " " + pair.friend1_last;
    let friend2_name = pair.friend2_first + " " + pair.friend2_last;
    prints(friend1_name + ", " + friend2_name);
  }
}