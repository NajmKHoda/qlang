# Closures

## Syntax

```rs
["failable"] "lambda"
"(" [QName ":" Typename {"," QName ":" Typename}] ")"
["->" Typename]
"{" ( Expression | {Statement} ) "}"
```

## Explanation

Closures are first-class callable values. They can be marked `failable`, can optionally declare a return type, and may use an expression body or statement body.

## Examples

```ql
function main() -> int {
    let twice: (int) -> int = lambda(x: int) { x * 2 };
    printi(twice(21));
    return 0;
}
```

Expected output:
```text
42
```

## See Also

- [Function and Method Calls](calls.md)
- [Callable Types](../types/callables.md)
- [Failable Semantics](../misc/failable-semantics.md)
