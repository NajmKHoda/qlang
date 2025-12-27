#include <stdio.h>
#include <stdbool.h>
#include "metadata.h"
#include "primitives.h"

QLTypeInfo __ql__int_type_info = {
    .size = sizeof(int),
    .elem_drop = NULL,
    .set_nth = NULL
};

QLTypeInfo __ql__bool_type_info = {
    .size = sizeof(bool),
    .elem_drop = NULL,
    .set_nth = NULL
};

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
