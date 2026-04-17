# Failable Semantics

## Syntax

```ql
failable function(...) { ... }
failable lambda(...) { ... }
failable (...) -> ...
```

## Explanation

`failable` annotates functions, closures, and callable types that may fail. In general, if a function or closure does one of the following outside of a `transaction` block:

- Call another function labeled as `failable`
- Invoke a closure that is typed as `failable`
- Use an immediate query (`query { ... }`)
- Evaluate a parameterized query (`query() { ... }`)

then it must be labeled as `failable`. This is similar to Java's `throws` semantics, but much simpler.

If a `failable` call runs into an error, control is returned to the topmost `on rollback` block on the execution stack and the associated transaction is rolled back, or the program exits if there is no such `on rollback` block (in which case `main` would be labeled as `failable`).

## Examples

```ql
failable function insert_student(y: str) -> void {
    let failable_closure = failable lambda(x: str) {
        query { insert { name: x } into Student };
    }
    failable_closure(y);
}
```

## See Also

- [Callable Types](../types/callables.md)
- [Closures](../expressions/closures.md)
- [Transaction Statement](../statements/transaction.md)
