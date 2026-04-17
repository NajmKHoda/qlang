# Builtin Functions

## Syntax

```ql
prints(str)
printi(int)
printd(float)
printb(bool)
inputs()
inputi()
zip(iter<T>, iter<T>)
concat(iter<T>, iter<T>)
```

## Explanation

Builtin functions are resolved by name during semantic analysis. Current builtin set includes I/O helpers (`print*`, `input*`) and iterator combinators (`zip`, `concat`).

## Examples

```ql
function main() -> int {
    prints("name?");
    let name: str = inputs();
    prints(name);
    return 0;
}
```

Expected output:
```text
name?
<echoed input>
```

## See Also

- [`array-methods.md`](array-methods.md)
- [`iterator-methods.md`](iterator-methods.md)
- [`../expressions/calls.md`](../expressions/calls.md)
