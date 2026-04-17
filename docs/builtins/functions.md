# Builtin Functions

## Syntax

```c++
void prints(str);
void printi(int);
void printd(float);
void printb(bool);

str inputs();
int inputi();

iter<T> zip(iter<T>, iter<T>);
iter<T> concat(iter<T>, iter<T>);
```

## Explanation

Current builtin set includes I/O helpers (`print*`, `input*`) and iterator combinators (`zip`, `concat`).

- The `print*()` family of builtins prints the respective datatype to standard output.
- The `input*()` family of builtins accepts the respective datatype from standard input and returns it.
- `zip()` returns a new iterator that alternates between the first and second given iterators.
    If one iterator runs out, it only takes from the other iterator.
- `concat()` returns a new iterator that moves through the first iterator, then the second after the first is exhausted.

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

- [Array Builtin Methods](array-methods.md)
- [Iterator Builtin Methods](iterator-methods.md)
- [Function and Method Calls](../expressions/calls.md)
