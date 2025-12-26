#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "metadata.h"
#include "array.h"

QLTypeInfo __ql__QLArray_type_info = {
    .size = sizeof(QLArray*),
    .elem_drop = __ql__QLArray_elem_drop
};

static inline void* __ql__QLArray_get_nth_elem(QLArray* array, unsigned int n) {
    return (char*)array->elems + (n * array->type_info->size);
}

QLArray* __ql__QLArray_new(void* elems, unsigned int num_elems, QLTypeInfo* type_info) {
    QLArray* array = malloc(sizeof(QLArray));
    array->num_elems = num_elems;
    array->type_info = type_info;
    array->ref_count = 1;

    // elem_capacity = max(8, next power of two)
    unsigned int capacity = num_elems;
    if (capacity < 8) {
        capacity = 8;
    } else {
        capacity |= capacity >> 1;
        capacity |= capacity >> 2;
        capacity |= capacity >> 4;
        capacity |= capacity >> 8;
        capacity |= capacity >> 16;
        capacity++;
    }

    array->capacity = capacity;
    array->elems = malloc(capacity * type_info->size);
    memcpy(array->elems, elems, num_elems * type_info->size);

    return array;
}

void __ql__QLArray_add_ref(QLArray* array) {
    array->ref_count++;
}

void __ql__QLArray_remove_ref(QLArray* array) {
    array->ref_count--;
    if (array->ref_count == 0) {
        if (array->type_info->elem_drop != NULL) {
            for (unsigned int i = 0; i < array->num_elems; i++) {
                void* elem_ptr = __ql__QLArray_get_nth_elem(array, i);
                array->type_info->elem_drop(elem_ptr);
            }
        }
        free(array->elems);
        free(array);
        fprintf(stderr, "free(array %p)\n", (void*)array);
    }
}

void __ql__QLArray_elem_drop(void* array_ptr) {
    __ql__QLArray_remove_ref(*(QLArray**)array_ptr);
}

void* __ql__QLArray_index(QLArray* array, unsigned int index) {
    if (index >= array->num_elems) {
        fprintf(stderr, "Array element index out of bounds (%u >= %u)\n", index, array->num_elems);
        exit(1);
    }
    return __ql__QLArray_get_nth_elem(array, index);
}
