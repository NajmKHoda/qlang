# Ranges

## Syntax

```ql
irange(<start>..<end>)
irange(<start>..=<end>)
irange(..<end>)
irange(<start>..)
irange(..)
irange(<start>..<end>:<step>)
```

## Explanation

`irange(...)` builds a range expression with optional start, end, inclusive end marker (`..=`), and optional step. Ranges are represented in IR as `Range { start, end, inclusive, step }`.

## Examples

```ql
function main() -> int {
    for i in irange(0..5) {
        printi(i);
    }
    return 0;
}
```

Expected output:
```text
0
1
2
3
4
```

## See Also

- [`../statements/loops.md`](../statements/loops.md)
- [`../types/collections.md`](../types/collections.md)
- [`../builtins/iterator-methods.md`](../builtins/iterator-methods.md)
