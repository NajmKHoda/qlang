# Arithmetic and Comparison

## Syntax

```rs
// Arithmetic operations
Expression "+" Expression // int, float, string
Expression "-" Expression // int, float
Expression "*" Expression // int, float
Expression "/" Expression // int, float
Expression "%" Expression // int

// Comparison operations
Expression "==" Expression
Expression "!=" Expression
Expression ">" Expression
Expression "<" Expression
Expression ">=" Expression
Expression "<=" Expression
```

## Explanation

If the type of both operands is `T`, arithmetic operators produce a value of type `T` (assuming `T` is supported for the operation). Comparison operators produce `bool`.

Addition with strings is defined as concatenation.

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

- [Logical Operators](logical-operators.md)
- [Primitive Type Conversions](type-conversions.md)
- [Conditional Statements](../statements/conditionals.md)
