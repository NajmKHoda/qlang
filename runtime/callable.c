#include <stdbool.h>

typedef struct {
    void* invoke_fn;
    void* captured_values;
} Callable;