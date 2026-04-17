# Delete Query

## Syntax

```ql
"delete" "from" ProperQName
["where" QName "==" Expression]
```

## Explanation

`delete` removes rows from a table, optionally filtered by a `where` clause.

## Examples

```ql
query {
    delete from Users where id == 1
};
```

## See Also

- [Select Query](select.md)
- [Update Query](update.md)
- [Transaction Statement](../statements/transaction.md)
