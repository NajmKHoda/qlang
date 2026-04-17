# Lone Expression Statements

## Syntax

```rs
Expression ";"
```

## Explanation

This simply evaluates the given expression and does nothing with the result. This is common for function/method calls and query expressions that are executed for side effects.

## Examples

```ql
function main() -> int {
    "dummy string";
    67;
    true;
    prints("hello from expression statement");
    return 0;
}
```

Expected output:
```text
hello from expression statement
```

## See Also

- [Function and Method Calls](../expressions/calls.md)
- [Query Expressions](../expressions/query-expressions.md)
- [Let Declarations and Assignment](let-and-assignment.md)
