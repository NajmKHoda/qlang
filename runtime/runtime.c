#include <stdio.h>
#include <stdbool.h>
#include <string.h>
#include <unistd.h>
#include <stdlib.h>
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

// Strings

typedef struct {
    char* raw_string;
    unsigned int length;
    unsigned int ref_count;
    bool is_global;
} QLString;

QLString* __ql__QLString_new(char* raw_string, int length, bool is_global) {
    QLString* result = malloc(sizeof(QLString));
    result->raw_string = raw_string;
    result->length = length;
    result->ref_count = 1;
    result->is_global = is_global;
    return result;
}

QLString* __ql__QLString_concat(QLString* a, QLString* b) {
    unsigned int length = a->length + b->length;
    char* raw_string = malloc(length);
    memcpy(raw_string, a->raw_string, a->length);
    memcpy(raw_string + a->length, b->raw_string, b->length);
    return __ql__QLString_new(raw_string, length, false);
}

int __ql__QLString_compare(QLString* a, QLString* b) {
    int an = a->length, bn = b->length;
    int n = (an < bn) ? an : bn;
    int cmp = memcmp(a->raw_string, b->raw_string, n);
    return (cmp != 0) ? cmp : (an - bn);
}

void __ql__QLString_add_ref(QLString* str) {
    str->ref_count++;
}

void __ql__QLString_remove_ref(QLString* str) {
    str->ref_count--;
    if (str->ref_count == 0) {
        if (!str->is_global) {
            free(str->raw_string);
        }
        free(str);
    }
}

void prints(QLString* str) {
    write(STDOUT_FILENO, str->raw_string, str->length);
    putchar('\n');
}

QLString* inputs() {
    size_t capacity = 16;
    char* buffer = malloc(capacity);

    char c = getchar();
    size_t i;
    for (i = 0; c != EOF && c != '\n'; i++) {
        if (i == capacity) {
            // Reallocate
            capacity <<= 1;
            char* new_buffer = malloc(capacity);
            memcpy(new_buffer, buffer, i);
            free(buffer);
            buffer = new_buffer;
        }
        buffer[i] = c;
        c = getchar();
    }

    return __ql__QLString_new(buffer, i, false);
}