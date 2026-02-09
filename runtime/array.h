#ifndef RUNTIME_ARRAY_H
#define RUNTIME_ARRAY_H

#include "metadata.h"

typedef struct {
    unsigned int num_elems;
    unsigned int capacity;
    unsigned int ref_count;
    QLTypeInfo* type_info;
    void* elems;
} QLArray;

extern QLTypeInfo __ql__QLArray_type_info;

QLArray* __ql__QLArray_new(void* elems, unsigned int num_elems, QLTypeInfo* type_info);
void __ql__QLArray_add_ref(QLArray* array);
void __ql__QLArray_remove_ref(QLArray* array);
void __ql__QLArray_elem_drop(void* array_ptr);
void* __ql__QLArray_index(QLArray* array, unsigned int index);

void __ql__QLArray_append(QLArray* array, void* elem_ptr);
int __ql__QLArray_length(QLArray* array);
void* __ql__QLArray_pop(QLArray* array);

#endif
