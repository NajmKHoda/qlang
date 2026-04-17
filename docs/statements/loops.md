# Loops

## Syntax

```rs
// While loop
"while" Expression "{" {Statement} "}" ["as" QName ";"]

// For loop
"for" QName "in" Expression "{" {Statement} "}" ["as" QName ";"]
```

## Explanation

QLang supports condition-based loops (`while`) and iteration-based loops (`for ... in`). Both forms can optionally define labels (`as label;`) used by `break label;` and `continue label;`.

- The `while` loop conditional must be of type `bool`.
- If the iterable expression in a `for` loop is of type `iter<T>` or `T[]`, the type of the loop variable is `T`. No other types are accepted as iterable.

## Examples

Example 1:
```js
function main() -> int {
    let i: int = 0;
    while i < 3 {
        printi(i);
        i = i + 1;
    }
    return 0;
}
```

Expected output:
```text
0
1
2
```

Example 2:
```js
function main() -> int {
    let arr: int[] = [ 1, 2, 3 ];
    for number in arr {
        printi(number);
    }
}
```

Expected output:
```
1
2
3
```

## See Also

- [`control-transfer.md`](control-transfer.md)
- [`../expressions/ranges.md`](../expressions/ranges.md)
- [`../types/collections.md`](../types/collections.md)
