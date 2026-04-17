# Datasources and Tables

## Syntax

```ql
["readonly"] datasource <name:QName>;
["readonly"] table <name:ProperQName> from <datasource_name:QName> { <columns:Comma<TypedQName>> }
```

## Explanation

Datasources define external storage handles. Tables map schema declarations onto a datasource and define typed columns. The optional `readonly` modifier marks declarations that should not be mutated.

## Examples

```ql
readonly datasource AppDb;

table Users from AppDb {
    id: int,
    name: str
}
```

Expected output:
```text
(schema declaration only; no stdout)
```

## See Also

- [`program-structure.md`](program-structure.md)
- [`../queries/select.md`](../queries/select.md)
- [`../queries/insert.md`](../queries/insert.md)
