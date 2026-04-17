# Qualified Columns and Aliases

## Syntax

```rs
ColumnRef ::= [ProperQName "."] QName
```

## Explanation

In `SELECT` statements, a column reference can be qualified by table alias (`Users.id`) or unqualified (`id`) depending on ambiguity and context. Table aliases introduced with `as` improve readability in joins and large queries.

## Examples

```ql
query {
    select all
    from Users as U
    join Orders as O on U.id == user_id
    where O.total == 100
};
```

## See Also

- [Select Query](select.md)
- [Query Expressions](../expressions/query-expressions.md)
- [Datasources and Tables](../misc/datasource-and-table.md)
