# Primitive Type Conversions

## Syntax

```ql
<int|float|bool|str>(<value:Expression>)
```

## Explanation

Primitive conversion syntax is function-like and represented in IR as `Conversion(value, target_type)` / semantic `Convert`. Use conversions when you want explicit intent around numeric and string coercions.

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
