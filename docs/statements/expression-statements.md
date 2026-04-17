# Expression Statements

## Syntax

```ql
<Expression> ";"
```

## Explanation

Any expression can be used as a statement when its result is not bound. This is common for function/method calls and query expressions that are executed for side effects.

## Examples

```ql
function main() -> int {
    prints("hello from expression statement");
    return 0;
}
```

Expected output:
```text
hello from expression statement
```

## See Also

- [`../expressions/calls.md`](../expressions/calls.md)
- [`../expressions/query-expressions.md`](../expressions/query-expressions.md)
- [`let-and-assignment.md`](let-and-assignment.md)
