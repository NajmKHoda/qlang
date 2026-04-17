# Naming and Identifiers

## Syntax

```ql
QName
ProperQName
```

## Explanation

QLang grammar distinguishes between general names (`QName`) and proper names (`ProperQName`) for constructs such as table/struct names. Follow consistent naming conventions to keep schemas and code readable.

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

- [`program-structure.md`](program-structure.md)
- [`../types/structs.md`](../types/structs.md)
- [`../queries/qcolumn-and-aliases.md`](../queries/qcolumn-and-aliases.md)
