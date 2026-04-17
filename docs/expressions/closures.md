# Closures

## Syntax

```ql
["failable"] lambda "(" <params:Comma<TypedQName>> ")"
["->" <TypeName>] "{" <ClosureBody> "}"
```

## Explanation

Closures are first-class callable values. They can be marked `failable`, can optionally declare a return type, and may use an expression body or statement body.

The semantic IR models closures as `Closure { closure_id, ... }` expressions with callable types.

## Examples

```ql
function main() -> int {
    let twice: (int) -> int = lambda (x: int) -> int { x * 2 };
    printi(twice(21));
    return 0;
}
```

Expected output:
```text
42
```

## See Also

- [`calls.md`](calls.md)
- [`../types/callables.md`](../types/callables.md)
- [`../misc/failable-semantics.md`](../misc/failable-semantics.md)
