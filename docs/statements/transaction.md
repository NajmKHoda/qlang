# Transaction Statement

## Syntax

```ql
transaction { <Statement*> }
on rollback { <Statement*> }
```

## Explanation

A transaction groups a body and an explicit rollback body. The semantic IR tracks transactions with dedicated IDs and rollback blocks, enabling generated code to recover from failed operations.

## Examples

```ql
function transfer() -> void {
    transaction {
        // debit source account
        // credit target account
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
