# Qualified Columns and Aliases

## Syntax

```ql
<QColumn> ::= <TableName> "." <column_name> | <column_name>
<TableAlias> ::= "as" <Alias>
```

## Explanation

A query column can be qualified (`Users.id`) or unqualified (`id`) depending on ambiguity and context. Table aliases introduced with `as` improve readability in joins and large queries.

## Examples

```ql
query {
    select all
    from Users as U
    join Orders as O on U.id == user_id
    where O.total == 100
};
```

Expected output:
```text
(no direct stdout unless printed)
```

## See Also

- [`select.md`](select.md)
- [`../expressions/query-expressions.md`](../expressions/query-expressions.md)
- [`../misc/datasource-and-table.md`](../misc/datasource-and-table.md)
