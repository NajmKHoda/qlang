# Let Declarations and Assignment

## Syntax

```ql
"let" <name:QName> <var_type:(":" <TypeName>)?> "=" <init_expr:Expression> ";"
<name:QName> "=" <expr:Expression> ";"
```

## Explanation

Use `let` to introduce a local variable. Type annotations are optional when the initializer is sufficient for inference, but explicit types are useful for readability and stricter intent. Assignment updates an existing variable.

See also: variable declarations in semantic IR (`VariableDeclaration`, `VariableAssignment`).

## Examples

```ql
function main() -> int {
    let x: int = 10;
    x = x + 5;
    printi(x);
    return 0;
}
```

Expected output:
```text
15
```

## See Also

- [`../types/primitives.md`](../types/primitives.md)
- [`../expressions/arithmetic-and-comparison.md`](../expressions/arithmetic-and-comparison.md)
- [`control-transfer.md`](control-transfer.md)
