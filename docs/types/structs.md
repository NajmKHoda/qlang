# Struct Types

## Syntax

```rs
"struct" ProperQName "{" QName ":" Typename{"," QName ":" Typename} "}"
```

## Explanation

Structs define named field collections and are used for domain data and query capture. Struct literals can be explicitly typed (`User { ... }`) or inferred in compatible contexts.

## Examples

```ql
struct User {
    id: int,
    name: str
}

function main() -> int {
    let u: User = { id: 1, name: "Ada" };
    printi(u.id);
    prints(u.name);
    return 0;
}
```

Expected output:
```text
1
Ada
```

## See Also

- [Select Query](../queries/select.md)
- [Insert Query](../queries/insert.md)
- [Literals, Struct Literals, and Arrays](../expressions/literals-and-collections.md)
