# Array Builtin Methods

## Syntax

```c++
int T[].length();

void T[].append(T);

T T[].pop();

iter<T> T[].iter();
```

## Explanation

Arrays expose builtin methods for size introspection, mutation, and iterator conversion.

- `Array.length()` returns the number of items in the array.
- `Array.append(x)` appends the value of expression `x` to the end of the array.
- `Array.pop()` removes the element at the end of the array and returns it.
- `Array.iter()` returns an iterator over the array's elements.

## Examples

```ql
function main() -> int {
    let nums: int[] = [1, 2];
    nums.append(3);
    printi(nums.length());
    nums.pop();
    printi(nums.length());
    return 0;
}
```

Expected output:
```text
3
2
```

## See Also

- [`iterator-methods.md`](iterator-methods.md)
- [`../types/collections.md`](../types/collections.md)
- [`../expressions/calls.md`](../expressions/calls.md)
