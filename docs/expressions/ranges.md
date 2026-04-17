# Ranges

## Syntax

```rs
"irange" "(" RangeEndpoints [":" c] ")"

RangeEndpoints ::=
    a ".." b  |  // [a, b)
    a "..=" b |  // [a, b]
    ".." b    |  // [0, b)
    a ".."    |  // [a, infinity)
    ".."         // [0, infinity)

a : Expression
b : Expression
c : Expression
```

## Explanation

`irange()` builds a range expression with optional start `a`, end `b`, inclusive end marker (`..=`), and optional step `c`. It has a type of `iter<int>` and can thus be treated as an iterator.

## Examples

```ql
function main() -> int {
    for i in irange(0..5) {
        printi(i);
    }
    return 0;
}
```

Expected output:
```text
0
1
2
3
4
```

## See Also

- [`../statements/loops.md`](../statements/loops.md)
- [`../types/collections.md`](../types/collections.md)
- [`../builtins/iterator-methods.md`](../builtins/iterator-methods.md)
