# Delete Query

## Syntax

```ql
delete from <table_name:ProperQName> [where <column_name> == <Expression>]
```

## Explanation

`delete` removes rows from a table, optionally filtered by a `where` clause.

## Examples

```ql
query {
    delete from Users where id == 1
};
```

Expected output:
```text
(no direct stdout unless printed)
```

## See Also

- [`select.md`](select.md)
- [`update.md`](update.md)
- [`../statements/transaction.md`](../statements/transaction.md)
