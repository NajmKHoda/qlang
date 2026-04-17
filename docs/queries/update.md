# Update Query

## Syntax

```ql
update <table_name:ProperQName> set <column_name> = <Expression>, ... [where <column_name> == <Expression>]
```

## Explanation

`update` mutates one or more columns for matching rows. Without `where`, it can affect all rows in the target table.

## Examples

```ql
query {
    update Users set name = "Grace" where id == 1
};
```

Expected output:
```text
(no direct stdout unless printed)
```

## See Also

- [`select.md`](select.md)
- [`delete.md`](delete.md)
- [`../statements/transaction.md`](../statements/transaction.md)
