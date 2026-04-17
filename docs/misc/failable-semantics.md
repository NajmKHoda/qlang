# Failable Semantics

## Syntax

```ql
failable function <name>(...) -> <TypeName> { ... }
failable lambda (...) -> <TypeName> { ... }
failable (<params>) -> <ret>
```

## Explanation

`failable` annotates functions, closures, and callable types that may fail. In semantic IR, failable call paths track extra bookkeeping (for example, error-related drops in closure/function call/query nodes).

## Examples

```ql
failable function parse_id(raw: str) -> int {
    // skeleton: parse and potentially fail
    return inputi();
}
```

Expected output:
```text
(depends on failure behavior and caller handling)
```

## See Also

- [`../types/callables.md`](../types/callables.md)
- [`../expressions/closures.md`](../expressions/closures.md)
- [`../statements/transaction.md`](../statements/transaction.md)
