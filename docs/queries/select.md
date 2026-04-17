# Select Query

## Syntax

```ql
select <StructName> { <field_alias:QColumn>, ... }
from <RootTable> [as <Alias>]
[join <RightTable> [as <Alias>] on <QColumn> == <column_name>]*
[where <QColumn> == <Expression>]
[limit <Expression>]
[offset <Expression>]

select all from <RootTable> [as <Alias>] ...
```

## Explanation

`select` supports explicit capture into a struct shape or `select all`. Query clauses include joins, where filters, limit, and offset.

In semantic IR this maps to `SemanticQuery::Select` with captured columns, table IDs, and optional count/filter clauses.

## Examples

```ql
let users_q = query {
    select all
    from Users as U
    where U.id == 1
    limit 1
};
```

Expected output:
```text
(no direct stdout unless consumed and printed)
```

## See Also

- [`qcolumn-and-aliases.md`](qcolumn-and-aliases.md)
- [`insert.md`](insert.md)
- [`../expressions/query-expressions.md`](../expressions/query-expressions.md)
