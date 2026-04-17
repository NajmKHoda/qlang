# Return, Break, and Continue

## Syntax

```ql
return <Expression?>;
break <QName?>;
continue <QName?>;
```

## Explanation

`return` exits the current function and may return a value. `break` and `continue` affect loop flow and may optionally target a labeled loop.

In semantic IR, these map to explicit control-flow statements (`Return`, `Break`, `Continue`).

## Examples

```ql
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
