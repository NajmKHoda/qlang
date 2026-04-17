# Naming and Identifiers

## Syntax

```rs
// Qualified names
QName ::= /_*[a-z][a-zA-Z0-9_]*/

// Proper qualified names
ProperQName ::= /_*[A-Z][a-zA-Z0-9]*/

```

## Explanation

QLang grammar distinguishes between general names (`QName`) and proper names (`ProperQName`) for constructs such as table/struct names. The first non-underscore character in `QName` must be lowercase, while it must be uppercase in `ProperQName`. Follow consistent naming conventions to keep schemas and code readable.

Suggested style:
- `ProperQName`: PascalCase for structs/tables.
- `QName`: snake_case for variables and parameters.

## Examples

```ql
struct UserProfile { user_id: int, display_name: str }

function load_profile(user_id: int) -> void {
    let display_name: str = "guest";
    prints(display_name);
}
```

Expected output:
```text
guest
```

## See Also

- [Program Structure](program-structure.md)
- [Struct Types](../types/structs.md)
- [Qualified Columns and Aliases](../queries/qcolumn-and-aliases.md)
