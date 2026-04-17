# Literals, Struct Literals, and Arrays

## Syntax

```ql
Int | Float | Bool | QLString
<name:ProperQName?> "{" <fields:Comma<ColumnValue>> "}"
"[" <Comma<Expression>> "]"
```

## Explanation

QLang literals include integers, floats, booleans, and strings. Struct literals can be named (`User { ... }`) or anonymous (`{ ... }`), and arrays use bracket syntax.

## Examples

```ql
function main() -> int {
    let ok: bool = true;
    let nums: int[] = [1, 2, 3];
    let p = { x: 10, y: 20 };

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

- [`../types/primitives.md`](../types/primitives.md)
- [`../types/collections.md`](../types/collections.md)
- [`calls.md`](calls.md)
