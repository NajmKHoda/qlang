#ifndef RUNTIME_ITERATOR_H
#define RUNTIME_ITERATOR_H

#include <stdbool.h>

typedef struct QLTypeInfo QLTypeInfo;
typedef struct QLArray QLArray;
typedef struct QLIterator QLIterator;

struct QLIterator {
    // Returns the pointer to the next element, or NULL if there are no more elements.
    void* (*next)(QLIterator* iterator); 
    bool (*has_next)(QLIterator* iterator);
    unsigned int index; // Internal state index.
    void* iterable;

    QLTypeInfo* iterable_type_info;
    QLTypeInfo* elem_type_info;
    unsigned int ref_count; // Reference count for memory management.
};

QLIterator* __ql__QLIterator_new(
    void* iterable,
    void* (*next_func)(QLIterator*),
    bool (*has_next_func)(QLIterator*),
    QLTypeInfo* iterable_type_info,
    QLTypeInfo* elem_type_info
);
void* __ql__QLIterator_next(QLIterator* iter);
bool __ql__QLIterator_has_next(QLIterator* iter);
QLArray* __ql__QLIterator_collect(QLIterator* iter);

void __ql__QLIterator_copy(QLIterator** iter_ptr);
void __ql__QLIterator_drop(QLIterator** iter_ptr);

#endif