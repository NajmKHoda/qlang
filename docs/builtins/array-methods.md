# Array Builtin Methods

## Syntax

```ql
<array_expr>.length()
<array_expr>.append(value)
<array_expr>.pop()
<array_expr>.iter()
```

## Explanation

Arrays expose builtin methods for size introspection, mutation, and iterator conversion. The semantic layer resolves these as builtin method calls.

## Examples

```ql
function main() -> int {
    let nums: int[] = [1, 2];
    nums.append(3);
    printi(nums.length());
    nums.pop();
    printi(nums.length());
    return 0;
}
```

Expected output:
```text
3
2
```

## See Also

- [`iterator-methods.md`](iterator-methods.md)
- [`../types/collections.md`](../types/collections.md)
- [`../expressions/calls.md`](../expressions/calls.md)
