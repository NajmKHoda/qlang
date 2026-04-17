# Program Structure

## Syntax

```rs
Program ::=
    {Datasource}
    {Table}
    {Struct}
    {Function}
```

## Explanation

A QLang program is composed of four types of declarations in this order:

- [Datasources](datasource-and-table.md): external storage handlers.
- [Tables](datasource-and-table.md): records within datasources.
- [Structs](../types/structs.md): records without associated datasources.
- [Functions](functions.md): executable code.

 All QLang programs must have a `main` function.

## Examples

```ql
readonly datasource AppDb;

table Users from AppDb { id: int, name: str }

struct User { id: int, name: str }

function main() -> int {
    return 0;
}
```

Expected output:
```text
(no output; exits with code 0)
```

## See Also

- [Datasources and Tables](datasource-and-table.md)
- [Struct Types](../types/structs.md)
- [Return, Break, and Continue](../statements/control-transfer.md)
