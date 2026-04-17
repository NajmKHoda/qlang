# Insert Query

## Syntax

```ql
"insert" Expression "into" ProperQName
```

## Explanation

`insert` takes an expression value and inserts it into a target table. If `T` is the target table's type, the expression can be of types `T` (for a single row) or `T[]` (for multiple rows at once).

## Examples

```ql
query {
    insert User { id: 1, name: "Ada" } into Users
};
```

## See Also

- [Update Query](update.md)
- [Delete Query](delete.md)
- [Struct Types](../types/structs.md)
