# Query Expressions

## Syntax

```ql
query { <Query> }
query(<parameters:Comma<TypedQName>>) { <Query> }
```

## Explanation

Queries are expressions, not standalone top-level declarations. QLang supports immediate queries and parameterized query expressions. These are represented in IR as `ImmediateQuery` and `ParameterizedQuery`.

## Examples

```ql
function get_all_users() -> void {
    let q = query {
        select all from Users
    };
    q;
}
```

Expected output:
```text
(no direct stdout unless printed)
```

## See Also

- [`../queries/select.md`](../queries/select.md)
- [`../queries/insert.md`](../queries/insert.md)
- [`../statements/expression-statements.md`](../statements/expression-statements.md)
