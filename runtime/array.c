#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "metadata.h"
#include "array.h"

QLTypeInfo __ql__QLArray_type_info = {
    .size = sizeof(QLArray*),
    .elem_drop = __ql__QLArray_elem_drop,
    .set_nth = NULL
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

void __ql__QLArray_append(QLArray* array, void* elem_ptr) {
    if (array->num_elems >= array->capacity) {
        unsigned int new_capacity = array->capacity * 2;
        void* new_elems = malloc(new_capacity * array->type_info->size);
        memcpy(new_elems, array->elems, array->num_elems * array->type_info->size);
        free(array->elems);
        array->elems = new_elems;
        array->capacity = new_capacity;
    }
    void* dest_ptr = __ql__QLArray_get_nth_elem(array, array->num_elems);
    memcpy(dest_ptr, elem_ptr, array->type_info->size);
    array->num_elems++;
}

int __ql__QLArray_length(QLArray* array) {
    return array->num_elems;
}

void* __ql__QLArray_pop(QLArray* array) {
    if (array->num_elems == 0) {
        fprintf(stderr, "Array.pop from empty array\n");
        exit(1);
    }

    unsigned int index = array->num_elems - 1;
    void* elem_ptr = __ql__QLArray_get_nth_elem(array, index);
    array->num_elems--;
    return elem_ptr;
}
