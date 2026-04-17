# Iterator Builtin Methods

## Syntax

```c++
bool iter<T>.has_next()

T iter<T>.next()

T[] iter<T>.collect()
```

## Explanation

Iterators support pull-based traversal and materialization into arrays.
These methods are key for loops and composable collection pipelines.

- `has_next()` returns `true` if and only if the iterator is not exhausted.
    It is a good idea to check this before every call to `next()`.
- `next()` returns the iterator's next element. If the iterator is exhausted, this will panic.
- `collect()` places the iterator's remaining elements into an array.
    This will always exhaust the iterator (unless it is infinite, in which case this will stall indefinitely).

## Examples

```ql
function main() -> int {
    let it = [10, 20, 30].iter();
    while it.has_next() {
        printi(it.next());
    }
    return 0;
}
```

Expected output:
```text
10
20
30
```

## See Also

- [Array Builtin Methods](array-methods.md)
- [Ranges](../expressions/ranges.md)
- [Loops](../statements/loops.md)
