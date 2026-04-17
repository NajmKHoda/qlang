# Lone Expression Statements

## Syntax

```rs
Expression ";"
```

## Explanation

This simply evaluates the given expression and does nothing with the result. This is common for function/method calls and query expressions that are executed for side effects.

## Examples

```js
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

- [`../expressions/calls.md`](../expressions/calls.md)
- [`../expressions/query-expressions.md`](../expressions/query-expressions.md)
- [`let-and-assignment.md`](let-and-assignment.md)
