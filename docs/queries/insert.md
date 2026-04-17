# Insert Query

## Syntax

```ql
insert <data_expr:Expression> into <table_name:ProperQName>
```

## Explanation

`insert` takes an expression value and inserts it into a target table. The inserted expression typically matches table column structure through named struct compatibility.

## Examples

```ql
query {
    insert User { id: 1, name: "Ada" } into Users
};
```

Expected output:
```text
(no direct stdout unless printed)
```

## See Also

- [`update.md`](update.md)
- [`delete.md`](delete.md)
- [`../types/structs.md`](../types/structs.md)
