# Update Query

## Syntax

```rs
"update" ProperQName "set"
QName "=" Expression{"," QName "=" Expression}
["where" QName "==" Expression]
```

## Explanation

`update` mutates one or more columns for matching rows. Without `where` to select specific rows, it affects all rows in the target table.

## Examples

```ql
query {
    update Users set
        name = "Grace"
        age = 20
    where id == 1
};
```

## See Also

- [Select Query](select.md)
- [Delete Query](delete.md)
- [Transaction Statement](../statements/transaction.md)
