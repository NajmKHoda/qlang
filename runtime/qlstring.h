#ifndef RUNTIME_STRING_H
#define RUNTIME_STRING_H

#include <stdbool.h>
#include "metadata.h"

extern QLTypeInfo __ql__QLString_type_info;

typedef struct QLString {
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

int __ql__str_to_int(QLString* str);
double __ql__str_to_float(QLString* str);
bool __ql__str_to_bool(QLString* str);
QLString* __ql__int_to_string(int x);
QLString* __ql__float_to_string(double x);
QLString* __ql__bool_to_string(bool x);

#endif
