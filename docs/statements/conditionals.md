# Conditional Statements

## Syntax

```ql
if <condition:Expression> { <Statement*> }
else if <condition:Expression> { <Statement*> }
else { <Statement*> }
```

## Explanation

QLang conditionals are statement-oriented and support chained `else if` branches plus an optional `else` fallback. Conditions must be boolean expressions.

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
