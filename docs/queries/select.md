# Select Query

## Syntax

```rs
"select" ("all" | ProperQName "{" QName ":" ColumnRef{"," QName ":" ColumnRef} "}")
"from" ProperQName ["as" ProperQName]
{"join" ProperQName ["as" ProperQName] "on" ColumnRef == QName}
["where" ColumnRef "==" Expression]
["limit" Expression]
["offset" Expression]
```

## Explanation

`select` retrieves rows from a database. It supports explicit capture into a struct shape, or `select all` to use the starting table struct for capture if no `join` clauses are present.

- `join` clauses perform a join operation.
- `where` filters out rows on a predicate (only equality is supported at the moment).
- `limit` constrains the maximum number of rows returned. The provided expression must be of type `int`.
- `offset` skips a number of rows at the start. The provided expression must be of type `int`.

## Examples

```ql
let users_q = query {
    select all
    from Users as U
    where U.id == 1
    limit 1
};
```

## See Also

- [Qualified Columns and Aliases](qcolumn-and-aliases.md)
- [Insert Query](insert.md)
- [Query Expressions](../expressions/query-expressions.md)
