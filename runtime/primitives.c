#include <stdio.h>
#include <stdbool.h>
#include "primitives.h"

void printi(int x) {
    printf("%d\n", x);
}

void printb(bool x) {
    printf("%s\n", x ? "true" : "false");
}

int inputi() {
    int x;
    scanf("%d", &x);
    while(getchar() != '\n');
    return x;
}
