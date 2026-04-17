# Transaction Statement

## Syntax

```rs
"transaction" "{" {Statement} "}"
"on" "rollback" "{" {Statement} "}"
```

## Explanation

A `transaction` statement represents an *atomic* (all-or-nothing) operation within a database.

If an error occurs within the main block, the transaction is "rolled back;" that is, the effects of all mutating queries within the main block are undone, and the database reverts to its state just before the `transaction` block began executing. Control is transferred immediately to the `on rollback` block.

Errors can be caused by immediate queries, parameterized queries, or calls to failable functions. See the [doc page on `failable` semantics](../misc/failable-semantics.md) for more information.

`transaction ... on rollback` is similar to `try ... catch` in other languages in that it acts as an error handling mechanism.

## Examples

```js
function transfer() -> void {
    transaction {
        // Let's assume this fails.
        query {
            insert { data: "foo" }
            into BadTable
        };
    } on rollback {
        // compensating actions
        prints("rollback executed");
    }
}
```

Expected output:
```text
rollback executed
```

## See Also

- [`../queries/update.md`](../queries/update.md)
- [`../misc/failable-semantics.md`](../misc/failable-semantics.md)
- [`control-transfer.md`](control-transfer.md)
