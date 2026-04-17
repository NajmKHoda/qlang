# Collection Types: Arrays and Iterators

## Syntax

```ql
<TypeName>[]
iter<TypeName>
```

## Explanation

Arrays are concrete collection values, while iterators represent sequential traversal. Builtin methods provide conversion and traversal (`iter`, `next`, `has_next`, `collect`).

## Examples

```ql
function main() -> int {
    let nums: int[] = [1, 2, 3];
    let it: iter<int> = nums.iter();
    while it.has_next() {
        printi(it.next());
    }
    return 0;
}
```

Expected output:
```text
1
2
3
```

## See Also

- [`primitives.md`](primitives.md)
- [`../builtins/array-methods.md`](../builtins/array-methods.md)
- [`../builtins/iterator-methods.md`](../builtins/iterator-methods.md)
