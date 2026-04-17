# Iterator Builtin Methods

## Syntax

```ql
<iter_expr>.has_next()
<iter_expr>.next()
<iter_expr>.collect()
```

## Explanation

Iterators support pull-based traversal and materialization into arrays. These methods are key for loops and composable collection pipelines.

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

- [`array-methods.md`](array-methods.md)
- [`../expressions/ranges.md`](../expressions/ranges.md)
- [`../statements/loops.md`](../statements/loops.md)
