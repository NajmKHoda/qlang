#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "array.h"

QLArray* __ql__QLArray_new(void* elems, unsigned int num_elems, unsigned long elem_size) {
    QLArray* array = malloc(sizeof(QLArray));
    array->num_elems = num_elems;
    array->elem_size = elem_size;
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
    array->elems = malloc(capacity * elem_size);
    memcpy(array->elems, elems, num_elems * elem_size);

    return array;
}

void __ql__QLArray_add_ref(QLArray* array) {
    array->ref_count++;
}

void __ql__QLArray_remove_ref(QLArray* array) {
    array->ref_count--;
    if (array->ref_count == 0) {
        free(array->elems);
        free(array);
    }
}

void* __ql__QLArray_index(QLArray* array, unsigned int index) {
    if (index >= array->num_elems) {
        fprintf(stderr, "Array element index out of bounds (%u >= %u)\n", index, array->num_elems);
        exit(1);
    }
    return (char*)array->elems + (index * array->elem_size);
}
