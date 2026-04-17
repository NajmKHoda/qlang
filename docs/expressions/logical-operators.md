# Logical Operators

## Syntax

```ql
not <Expression>
<Expression> and <Expression>
<Expression> or <Expression>
```

## Explanation

Logical operators compose boolean expressions used in conditionals, loops, and query filters. In IR, these map to `LogicalNot`, `LogicalAnd`, and `LogicalOr`.

## Examples

```ql
function main() -> int {
    let a: bool = true;
    let b: bool = false;

    printb(a and b);
    printb(a or b);
    printb(not b);

    return 0;
}
```

Expected output:
```text
false
true
true
```

## See Also

- [`arithmetic-and-comparison.md`](arithmetic-and-comparison.md)
- [`../statements/conditionals.md`](../statements/conditionals.md)
- [`../queries/select.md`](../queries/select.md)
