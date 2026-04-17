# Literals, Struct Literals, and Arrays

## Syntax

```rs
// Integer literal (no whitespace)
["+"|"-"] Digit {Digit}

// Float literal (no whitespace)
["+"|"-"] Digit {Digit} "." Digit {Digit}

// Bool literal
"true" | "false"

// String literal
'"' ... '"'

// Struct literal
[ProperQName] "{" [QName ":" Expression {"," QName ":" Expression}] "}"

// Array literal
"[" {Expression} "]"

Digit ::= "0" | "1" | "2" | ... | "9"
```

## Explanation

QLang literals include integers, floats, booleans, and strings. Struct literals can be named (`User { ... }`) or anonymous (`{ ... }`) if context is sufficient to determine the name of the struct. Arrays use bracket syntax.

## Examples

```ql
struct Point {
    x: int,
    y: int
}

function main() -> int {
    let ok: bool = true;
    let nums: int[] = [1, 2, 3];
    let p: Point = { x: 10, y: 20 };

    printb(ok);
    printi(nums.length());
    printi(p.x);
    return 0;
}
```

Expected output:
```text
true
3
10
```

## See Also

- [Primitive Types](../types/primitives.md)
- [Collection Types: Arrays and Iterators](../types/collections.md)
- [Function and Method Calls](calls.md)
