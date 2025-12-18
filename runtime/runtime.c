#include <stdio.h>
#include <stdbool.h>
#include "runtime.h"

void printi(int x) {
    printf("%d\n", x);
}

void printb(bool x) {
    printf("%s\n", x ? "true" : "false");
}

int inputi() {
    int x;
    scanf("%d", &x);
    return x;
}