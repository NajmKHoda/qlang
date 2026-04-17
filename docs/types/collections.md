# Collection Types: Arrays and Iterators

## Syntax

```rs
// Array type
TypeName "[]"

// Iterator type
"iter" "<" TypeName ">"
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

- [Primitive Types](primitives.md)
- [Array Builtin Methods](../builtins/array-methods.md)
- [Iterator Builtin Methods](../builtins/iterator-methods.md)
