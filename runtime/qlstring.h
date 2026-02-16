#ifndef RUNTIME_STRING_H
#define RUNTIME_STRING_H

#include <stdbool.h>
#include "metadata.h"

extern QLTypeInfo __ql__QLString_type_info;

typedef struct {
    char* raw_string;
    unsigned int length;
    unsigned int ref_count;
    bool is_global;
} QLString;

// String functions
QLString* __ql__QLString_new(char* raw_string, int length, bool is_global);
QLString* __ql__QLString_concat(QLString* a, QLString* b);
int __ql__QLString_compare(QLString* a, QLString* b);
void __ql__QLString_copy(QLString** str_ptr);
void __ql__QLString_drop(QLString** str_ptr);
void prints(QLString* str);
QLString* inputs();

#endif
