#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <ctype.h>
#include <stdbool.h>
#include "metadata.h"
#include "qlstring.h"

QLTypeInfo __ql__QLString_type_info = {
    .size = sizeof(QLString*),
    .copy = (void (*)(void*)) __ql__QLString_copy,
    .drop = (void (*)(void*)) __ql__QLString_drop
};

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

void __ql__QLString_copy(QLString** str_ptr) {
    QLString* str = *str_ptr;
    str->ref_count++;
}

void __ql__QLString_drop(QLString** str_ptr) {
    QLString* str = *str_ptr;
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
            capacity <<= 1;
            buffer = realloc(buffer, capacity);
        }
        buffer[i] = c;
        c = getchar();
    }

    return __ql__QLString_new(buffer, i, false);
}

int __ql__str_to_int(QLString* str) {
    char* raw = str->raw_string;
    int i = 0, n = str->length;

    // Determine sign (if any)
    bool is_negative = false;
    if (i < n && str->raw_string[0] == '-') {
        is_negative = true;
        i++;
    } else if (i < n && str->raw_string[0] == '+') {
        i++;
    }

    // Parse digits
    int result = 0;
    for (; i < n; i++) {
        char c = str->raw_string[i];
        if (c < '0' || c > '9') {
            return 0; // Invalid integer string
        }
        result = result * 10 + (c - '0');
    }

    return is_negative ? -result : result;
}

double __ql__str_to_float(QLString* str) {
    if (str->length == 0) {
        return 0.0;
    }

    // Create a c-string to use with stdlib
    char* cstr = malloc(str->length + 1);
    memcpy(cstr, str->raw_string, str->length);
    cstr[str->length] = '\0';

    char* endptr;
    double value = strtod(cstr, NULL);
    free(cstr);
    return value;
}

bool __ql__str_to_bool(QLString* str) {
    return str->length != 0;
}

QLString* __ql__int_to_string(int x) {
    char* raw;
    int len = asprintf(&raw, "%d", x);
    return __ql__QLString_new(raw, len, false);
}

QLString* __ql__float_to_string(double x) {
    char* raw;
    int len = asprintf(&raw, "%.3g", x);
    return __ql__QLString_new(raw, len, false);
}

QLString* __ql__bool_to_string(bool x) {
    if (x) {
        char* raw = malloc(4);
        memcpy(raw, "true", 4);
        return __ql__QLString_new(raw, 4, false);
    }

    char* raw = malloc(5);
    memcpy(raw, "false", 5);
    return __ql__QLString_new(raw, 5, false);
}
