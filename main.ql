table Point {
  int x,
  int y
}

function genPoint() -> Point {
  return Point { x: 0, y: 0 };
}

function main() -> int {
  Point point <- genPoint();
}