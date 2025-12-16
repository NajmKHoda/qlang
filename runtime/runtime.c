#include <stdio.h>
#include "runtime.h"

void printi(int x) {
    printf("%d\n", x);
}

int inputi() {
    int x;
    scanf("%d", &x);
    return x;
}