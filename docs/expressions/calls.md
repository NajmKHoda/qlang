# Function and Method Calls

## Syntax

```ql
<QName> "(" <Comma<Expression>> ")"
<Expression> "." <QName> "(" <Comma<Expression>> ")"
<Expression> "." <QName>
<Expression> "[" <Expression> "]"
```

## Explanation

QLang supports direct function calls, method calls on receivers, field access, and indexing expressions. Methods include builtin array/iterator methods and can be chained.

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
