# Function Declarations

## Syntax

```rs
["failable"] "function" QName "(" [QName ":" Typename{"," QName ":" Typename}] ")"
"->" Typename
"{" {Statement} "}"
```

## Explanation

Functions are top-level declarations with explicit return types. `main` is the conventional entrypoint and must have the signature `function main() -> int`.

Functions can call builtins, execute queries, and use all statement/expression forms.

## Examples

```ql
function add(a: int, b: int) -> int {
    return a + b;
}

function main() -> int {
    printi(add(20, 22));
    return 0;
}
```

Expected output:
```text
42
```

## See Also

- [Callable Types](../types/callables.md)
- [Function and Method Calls](../expressions/calls.md)
- [Return, Break, and Continue](../statements/control-transfer.md)
