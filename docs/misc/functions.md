# Function Declarations

## Syntax

```ql
["failable"] function <name:QName>
"(" <params:Comma<TypedQName>> ")"
"->" <return_type:TypeName>
"{" <body:Statement*> "}"
```

## Explanation

Functions are top-level declarations with explicit return types. `main` is the conventional entrypoint and is validated by semantic analysis (signature constraints are enforced).

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

- [`../types/callables.md`](../types/callables.md)
- [`../expressions/calls.md`](../expressions/calls.md)
- [`../statements/control-transfer.md`](../statements/control-transfer.md)
