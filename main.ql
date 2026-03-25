datasource data;

table Person from data {
  age: int,
  name: str,
  occupation: str
}

function main() -> int {
  let odds = [1, 3, 5, 7, 9].iter();
  let evens = [0, 2, 4, 6, 8].iter();
  let concat_iter = concat(evens, odds);
  for x in concat_iter {
    printi(x);
  }
}