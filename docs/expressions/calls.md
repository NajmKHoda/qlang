# Function and Method Calls

## Syntax

```rs
// Function call
QName "(" [Expression{"," Expression}] ")"

// Method calls
Expression "." QName "(" [Expression{"," Expression}] ")"

// Field access
Expression "." QName

// Array indexing
Expression "[" Expression "]"
```

## Explanation

QLang supports:
- Direct function calls
- Indirect calls on closure expressions
- Method calls on receivers
- Field access on structs
- Indexing expressions for arrays.

Methods include builtin array/iterator methods and can be sometimes be chained.

## Examples

```ql
function main() -> int {
    let nums: int[] = [5, 6, 7];
    printi(nums.length());
    printi(nums[1]);
    return 0;
}
```

Expected output:
```text
3
6
```

## See Also

- [`../builtins/functions.md`](../builtins/functions.md)
- [`../builtins/array-methods.md`](../builtins/array-methods.md)
- [`literals-and-collections.md`](literals-and-collections.md)
