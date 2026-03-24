#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "metadata.h"
#include "iterator.h"
#include "array.h"

QLTypeInfo __ql__QLArray_type_info = {
    .size = sizeof(QLArray*),
    .copy = (void (*)(void*)) __ql__QLArray_copy,
    .drop = (void (*)(void*)) __ql__QLArray_drop
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

void __ql__QLArray_copy(QLArray** array_ptr) {
    QLArray* array = *array_ptr;
    array->ref_count++;
}

void __ql__QLArray_drop(QLArray** array_ptr) {
    QLArray* array = *array_ptr;
    array->ref_count--;
    if (array->ref_count == 0) {
        if (array->type_info->drop != NULL) {
            for (unsigned int i = 0; i < array->num_elems; i++) {
                void* elem_ptr = __ql__QLArray_get_nth_elem(array, i);
                array->type_info->drop(elem_ptr);
            }
        }
        free(array->elems);
        free(array);
        fprintf(stderr, "free(array %p)\n", (void*)array);
    }
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
        array->capacity *= 2;
        array->elems = realloc(array->elems, array->capacity * array->type_info->size);
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

static void* __ql__QLArray_iter_next(QLIterator* iter) {
    ArrayIteratorState* state = (ArrayIteratorState*)(iter->state);
    if (state->index >= state->array->num_elems) {
        return NULL;
    }

    void* next_elem = __ql__QLArray_get_nth_elem(state->array, state->index);
    state->index++;
    return next_elem;
}

static bool __ql__QLArray_iter_has_next(QLIterator* iter) {
    ArrayIteratorState* state = (ArrayIteratorState*)(iter->state);
    return state->index < state->array->num_elems;
}

static void __ql__QLArray_iter_drop(QLIterator* iter) {
    ArrayIteratorState* state = (ArrayIteratorState*)(iter->state);
    __ql__QLArray_drop(&state->array);
    free(iter);
}

QLIterator* __ql__QLArray_iter(QLArray* array) {
    QLIterator* iter = __ql__QLIterator_new(
        __ql__QLArray_iter_next,
        __ql__QLArray_iter_has_next,
        __ql__QLArray_iter_drop,
        sizeof(ArrayIteratorState),
        array->type_info
    );
    ArrayIteratorState* state = (ArrayIteratorState*)(iter->state);
    state->array = array;
    state->index = 0;
    return iter;
}
