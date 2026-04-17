# Callable Types

## Syntax

```ql
["failable"] "(" <params:Comma<TypeName>> ")" "->" <ret:TypeName>
```

## Explanation

Callable types describe function/closure signatures. The `failable` modifier marks callables that may fail and require failure-aware handling in semantic/codegen phases.

## Examples

```ql
function apply_twice(f: (int) -> int, x: int) -> int {
    return f(f(x));
}

function main() -> int {
    let plus_one: (int) -> int = lambda (n: int) -> int { n + 1 };
    printi(apply_twice(plus_one, 10));
    return 0;
}
```

Expected output:
```text
12
```

## See Also

- [`../expressions/closures.md`](../expressions/closures.md)
- [`../misc/failable-semantics.md`](../misc/failable-semantics.md)
- [`primitives.md`](primitives.md)
