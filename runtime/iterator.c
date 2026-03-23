#include <stdbool.h>
#include <stdlib.h>
#include "metadata.h"
#include "array.h"
#include "iterator.h"

QLIterator* __ql__QLIterator_new(
    void* iterable,
    void* (*next_func)(QLIterator*),
    bool (*has_next_func)(QLIterator*),
    QLTypeInfo* iterable_type_info,
    QLTypeInfo* elem_type_info
) {
    QLIterator* iter = malloc(sizeof(QLIterator));
    iter->iterable = iterable;
    iter->next = next_func;
    iter->has_next = has_next_func;
    iter->iterable_type_info = iterable_type_info;
    iter->elem_type_info = elem_type_info;
    iter->index = 0;
    iter->ref_count = 1;

    // Ensures that the iterable remains valid for the iterator lifetime
    if (iterable_type_info->copy != NULL) {
        iterable_type_info->copy(&iter->iterable);
    }

    return iter;
}

void* __ql__QLIterator_next(QLIterator* iterator) {
    return iterator->next(iterator);
}

bool __ql__QLIterator_has_next(QLIterator* iterator) {
    return iterator->has_next(iterator);
}

QLArray* __ql__QLIterator_collect(QLIterator* iter) {
    // Create a new array to collect elements
    QLArray* array = __ql__QLArray_new(NULL, 0, iter->elem_type_info);

    // Iterate through the elements and append them to the array
    void* elem;
    while (__ql__QLIterator_has_next(iter)) {
        elem = __ql__QLIterator_next(iter);
        __ql__QLArray_append(array, elem);
    }

    return array;
}

void __ql__QLIterator_copy(QLIterator** iter_ptr) {
    QLIterator* iter = *iter_ptr;
    iter->ref_count++;
}

void __ql__QLIterator_drop(QLIterator** iter_ptr) {
    QLIterator* iter = *iter_ptr;
    iter->ref_count--;
    if (iter->ref_count == 0) {
        if (iter->iterable_type_info->drop != NULL) {
            iter->iterable_type_info->drop(&iter->iterable);
        }
        free(iter);
    }
}


