# Let Declarations and Assignment

## Syntax

```rs
// Variable declaration
"let" QName [":" TypeName] "=" Expression ";"

// Variable assignment
QName "=" Expression ";"
```

## Explanation

Use `let` to introduce a local variable. Type annotations are optional when the initializer is sufficient for inference of a concrete type, but explicit types are useful for readability and stricter intent. Assignment updates an existing variable.



## Examples

Example 1:
```js
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

Example 2:
```js
function main() -> int {
    let x = [];
}
```

Expected error:
```text
Variable x has an ambiguous type: any[]
```

## See Also

- [`../types/primitives.md`](../types/primitives.md)
- [`../expressions/arithmetic-and-comparison.md`](../expressions/arithmetic-and-comparison.md)
- [`control-transfer.md`](control-transfer.md)
