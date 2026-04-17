# Primitive Type Conversions

## Syntax

```rs
("int" | "float" | "bool" | "str") "(" Expression ")"
```

## Explanation

QLang only supports explicit conversions from a primitive datatype to another primitive datatype. Conversion syntax is function-like. QLang does not support implicit coercions. A summary table of conversion rules can be found below:

|      | to `int` | to `float` | to `bool` | to `string` |
|------|--------|---------|--------|-----------|
| from `int`| Identity | `x` -> `x.0` | `0` -> `false`, otherwise `true`  | Decimal representation |
| from `float` | Round | Identity | `0.0` -> `false`, otherwise `true`| Decimal representation |
| from `bool` | `true` -> `1`, `false` -> `0`| `true` -> `1.0`, `false` -> `0.0` | Identity | `true` -> `"true"`, `false` -> `"false"`|
| from `string` | `0` if parsing fails, otherwise decimal-parsed integer | `0.0` if parsing fails, otherwise decimal-parsed float | `""` -> `false`, otherwise `true` | Identity |

## Examples

```ql
function main() -> int {
    let x: int = 42;
    let y: float = float(x);
    printd(y);
    return 0;
}
```

Expected output:
```text
42.0
```

## See Also

- [`arithmetic-and-comparison.md`](arithmetic-and-comparison.md)
- [`../types/primitives.md`](../types/primitives.md)
- [`../statements/let-and-assignment.md`](../statements/let-and-assignment.md)
