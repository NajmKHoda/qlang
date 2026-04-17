# Logical Operators

## Syntax

```rs
// Logical NOT
"not" Expression

// Logical AND
Expression "and" Expression

// Logical OR
Expression "or" Expression
```

## Explanation

Logical operators compose boolean expressions used in conditionals, loops, and query filters.

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

- [Arithmetic and Comparison](arithmetic-and-comparison.md)
- [Conditional Statements](../statements/conditionals.md)
- [Select Query](../queries/select.md)
