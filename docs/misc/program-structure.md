# Program Structure

## Syntax

```ql
Program := Datasource* Table* Struct* Function*
```

## Explanation

A QLang program is composed of declarations followed by function definitions. The parser models this as `ProgramNode` with four ordered groups: datasources, tables, structs, and functions.

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

- [`datasource-and-table.md`](datasource-and-table.md)
- [`../types/structs.md`](../types/structs.md)
- [`../statements/control-transfer.md`](../statements/control-transfer.md)
