# Datasources and Tables

## Syntax

```rs
// Datasources
["readonly"] "datasource" QName;

// Tables
["readonly"] "table" ProperQName "from" QName
"{" QName ":" Typename{"," QName ":" Typename} "}"
```

## Explanation

Datasources define external storage handles. Tables map schema declarations onto a datasource and define typed columns.

- Table declarations automatically get their own struct type.
- The optional `readonly` modifier marks declarations that should not be mutated by `insert`, `update`, or `delete` queries. If a table originates from a `readonly` datasource, it must also be marked `readonly`.

## Examples

```ql
readonly datasource AppDb;

readonly table Users from AppDb {
    id: int,
    name: str
}
```

## See Also

- [Program Structure](program-structure.md)
- [Select Query](../queries/select.md)
- [Insert Query](../queries/insert.md)
