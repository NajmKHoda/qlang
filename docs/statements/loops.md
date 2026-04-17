# Loops

## Syntax

```ql
while <condition:Expression> { <Statement*> } [as <QName>;]
for <variable_name:QName> in <iterable_expr:Expression> { <Statement*> } [as <QName>;]
```

## Explanation

QLang supports condition-based loops (`while`) and iteration-based loops (`for ... in`). Both forms can optionally define labels (`as label;`) used by `break label;` and `continue label;`.

## Examples

```ql
function main() -> int {
    let i: int = 0;
    while i < 3 {
        printi(i);
        i = i + 1;
    }
    return 0;
}
```

Expected output:
```text
0
1
2
```

## See Also

- [`control-transfer.md`](control-transfer.md)
- [`../expressions/ranges.md`](../expressions/ranges.md)
- [`../types/collections.md`](../types/collections.md)
