# Arithmetic and Comparison

## Syntax

```ql
<Expression> "+" <Expression>
<Expression> "-" <Expression>
<Expression> "*" <Expression>
<Expression> "/" <Expression>
<Expression> "%" <Expression>
<Expression> "==" <Expression>
<Expression> "!=" <Expression>
<Expression> ">" <Expression>
<Expression> "<" <Expression>
<Expression> ">=" <Expression>
<Expression> "<=" <Expression>
```

## Explanation

Arithmetic operators produce numeric results. Comparison operators produce `bool`. Operator precedence is defined in the grammar and reflected directly in expression IR variants such as `Add`, `Multiply`, and `Compare`.

## Examples

```ql
function main() -> int {
    let a: int = 9;
    let b: int = 4;

    printi(a + b);
    printi(a % b);
    printb(a > b);

    return 0;
}
```

Expected output:
```text
13
1
true
```

## See Also

- [`logical-operators.md`](logical-operators.md)
- [`type-conversions.md`](type-conversions.md)
- [`../statements/conditionals.md`](../statements/conditionals.md)
