# Struct Types

## Syntax

```ql
struct <name:ProperQName> { <fields:Comma<TypedQName>> }
<TypeName> ::= <ProperQName>
```

## Explanation

Structs define named field collections and are used for domain data and query capture. Struct literals can be explicitly typed (`User { ... }`) or inferred in compatible contexts.

## Examples

```ql
struct User { id: int, name: str }

function main() -> int {
    let u: User = User { id: 1, name: "Ada" };
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

- [`../queries/select.md`](../queries/select.md)
- [`../queries/insert.md`](../queries/insert.md)
- [`../expressions/literals-and-collections.md`](../expressions/literals-and-collections.md)
