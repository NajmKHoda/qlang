#ifndef RUNTIME_ITERATOR_H
#define RUNTIME_ITERATOR_H

#include <stdbool.h>
#include <stddef.h>

typedef struct QLTypeInfo QLTypeInfo;
typedef struct QLArray QLArray;
typedef struct QLIterator QLIterator;

struct QLIterator {
    void* (*next)(QLIterator* iterator); 
    bool (*has_next)(QLIterator* iterator);
    void (*drop)(QLIterator* iterator_ptr);
    QLTypeInfo* elem_type_info;
    unsigned int ref_count;

    // Flexible state member
    size_t state_size;
    unsigned char state[];
};

QLIterator* __ql__QLIterator_new(
    void* (*next_func)(QLIterator*),
    bool (*has_next_func)(QLIterator*),
    void (*drop_func)(QLIterator*),
    size_t state_size,
    QLTypeInfo* elem_type_info
);
void* __ql__QLIterator_next(QLIterator* iter);
bool __ql__QLIterator_has_next(QLIterator* iter);
QLArray* __ql__QLIterator_collect(QLIterator* iter);

void __ql__QLIterator_copy(QLIterator** iter_ptr);
void __ql__QLIterator_drop(QLIterator** iter_ptr);

#endif