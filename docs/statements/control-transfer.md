# Return, Break, and Continue

## Syntax

```rs
// Return
"return" [Expression] ";"

// Break
"break" [QName] ";"

// Continue
"continue" [QName] ";"
```

## Explanation

`return` exits the current function. A return expression is supplied *if and only if* the function's return type is non-`void`.

`break` exits the loop with the given label, or the innermost loop if no label is provided. This is only valid within a loop.

`continue` immediately moves to the next iteration of the loop with the given label, or the innermost loop if no label is provided. This is only valid within a loop.

## Examples

```js
function main() -> int {
    let i: int = 0;
    while i < 10 {
        i = i + 1;
        if i == 3 {
            continue outer;
        }
        if i == 5 {
            break outer;
        }
        printi(i);
    } as outer;
    return 0;
}
```

Expected output:
```text
1
2
4
```

## See Also

- [`loops.md`](loops.md)
- [`conditionals.md`](conditionals.md)
- [`../misc/program-structure.md`](../misc/program-structure.md)
