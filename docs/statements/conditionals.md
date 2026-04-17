# Conditional Statements

## Syntax

```rs
"if" Expression "{" {Statement} "}"
{ "else" "if" Expression "{" {Statement} "}" }
[ "else" "{" {Statement} "}" ]
```

## Explanation

QLang conditionals are C-like and support chained `else if` branches plus an optional `else` fallback.
- Condition expressions must be of type `bool`.
- No parenthesis are required for the condition expression.

## Examples

```ql
function main() -> int {
    let n: int = 7;

    if n % 2 == 0 {
        prints("even");
    } else if n > 10 {
        prints("large odd");
    } else {
        prints("small odd");
    }

    return 0;
}
```

Expected output:
```text
small odd
```

## See Also

- [`loops.md`](loops.md)
- [`../expressions/logical-operators.md`](../expressions/logical-operators.md)
- [`../expressions/arithmetic-and-comparison.md`](../expressions/arithmetic-and-comparison.md)
