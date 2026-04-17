# Primitive Types

## Syntax

```rs
int
float
bool
str
void
```

## Explanation

Primitives are the foundation of QLang's type system. `void` is used for functions that do not return a value.

## Examples

```ql
function main() -> int {
    let i: int = 1;
    let f: float = 2.5;
    let b: bool = true;
    let s: str = "hi";

    printi(i);
    printd(f);
    printb(b);
    prints(s);
    return 0;
}
```

Expected output:
```text
1
2.5
true
hi
```

## See Also

- [Collection Types: Arrays and Iterators](collections.md)
- [Callable Types](callables.md)
- [Primitive Type Conversions](../expressions/type-conversions.md)
