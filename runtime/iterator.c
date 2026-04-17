#include <stdbool.h>
#include <stdlib.h>
#include <stddef.h>
#include <stdio.h>
#include <limits.h>
#include "metadata.h"
#include "primitives.h"
#include "array.h"
#include "iterator.h"

QLIterator* __ql__QLIterator_new(
    void* (*next_func)(QLIterator*),
    bool (*has_next_func)(QLIterator*),
    void (*drop_func)(QLIterator*),
    size_t state_size,
    QLTypeInfo* elem_type_info
) {
    QLIterator* iter = malloc(sizeof(QLIterator) + state_size);
    iter->next = next_func;
    iter->has_next = has_next_func;
    iter->drop = drop_func;
    iter->elem_type_info = elem_type_info;
    iter->ref_count = 1;
    iter->state_size = state_size;

    return iter;
}

void* __ql__QLIterator_next(QLIterator* iterator) {
    void* next = iterator->next(iterator);
    if (next == NULL) {
        fprintf(stderr, "next() called on exhausted iterator\n");
        exit(1);
    }
    return next;
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
        if (iter->drop != NULL) {
            iter->drop(iter);
        } else {
            free(iter);
        }
    }
}

static void* __ql__QLIterator_zip_next(QLIterator* iter) {
    ZipIteratorState* state = (ZipIteratorState*)(iter->state);

    if (state->next_from_a) {
        if (__ql__QLIterator_has_next(state->iter_a)) {
            state->next_from_a = false;
            return __ql__QLIterator_next(state->iter_a);
        }
        if (__ql__QLIterator_has_next(state->iter_b)) {
            return __ql__QLIterator_next(state->iter_b);
        }
        return NULL;
    }

    if (__ql__QLIterator_has_next(state->iter_b)) {
        state->next_from_a = true;
        return __ql__QLIterator_next(state->iter_b);
    }
    if (__ql__QLIterator_has_next(state->iter_a)) {
        return __ql__QLIterator_next(state->iter_a);
    }
    return NULL;
}

static bool __ql__QLIterator_zip_has_next(QLIterator* iter) {
    ZipIteratorState* state = (ZipIteratorState*)(iter->state);
    return __ql__QLIterator_has_next(state->iter_a) || __ql__QLIterator_has_next(state->iter_b);
}

static void __ql__QLIterator_zip_drop(QLIterator* iter) {
    ZipIteratorState* state = (ZipIteratorState*)(iter->state);
    __ql__QLIterator_drop(&state->iter_a);
    __ql__QLIterator_drop(&state->iter_b);
    free(iter);
}

QLIterator* __ql__QLIterator_zip(QLIterator* iter_a, QLIterator* iter_b) {
    QLIterator* iter = __ql__QLIterator_new(
        __ql__QLIterator_zip_next,
        __ql__QLIterator_zip_has_next,
        __ql__QLIterator_zip_drop,
        sizeof(ZipIteratorState),
        iter_a->elem_type_info
    );

    ZipIteratorState* state = (ZipIteratorState*)(iter->state);
    state->iter_a = iter_a;
    state->iter_b = iter_b;
    state->next_from_a = true;
    __ql__QLIterator_copy(&state->iter_a);
    __ql__QLIterator_copy(&state->iter_b);

    return iter;
}

static void* __ql__QLIterator_concat_next(QLIterator* iter) {
    ConcatIteratorState* state = (ConcatIteratorState*)(iter->state);

    if (state->using_a) {
        if (__ql__QLIterator_has_next(state->iter_a)) {
            return __ql__QLIterator_next(state->iter_a);
        }
        state->using_a = false;
    }

    if (__ql__QLIterator_has_next(state->iter_b)) {
        return __ql__QLIterator_next(state->iter_b);
    }

    return NULL;
}

static bool __ql__QLIterator_concat_has_next(QLIterator* iter) {
    ConcatIteratorState* state = (ConcatIteratorState*)(iter->state);
    return __ql__QLIterator_has_next(state->iter_a) || __ql__QLIterator_has_next(state->iter_b);
}

static void __ql__QLIterator_concat_drop(QLIterator* iter) {
    ConcatIteratorState* state = (ConcatIteratorState*)(iter->state);
    __ql__QLIterator_drop(&state->iter_a);
    __ql__QLIterator_drop(&state->iter_b);
    free(iter);
}

QLIterator* __ql__QLIterator_concat(QLIterator* iter_a, QLIterator* iter_b) {
    QLIterator* iter = __ql__QLIterator_new(
        __ql__QLIterator_concat_next,
        __ql__QLIterator_concat_has_next,
        __ql__QLIterator_concat_drop,
        sizeof(ConcatIteratorState),
        iter_a->elem_type_info
    );

    ConcatIteratorState* state = (ConcatIteratorState*)(iter->state);
    state->iter_a = iter_a;
    state->iter_b = iter_b;
    state->using_a = true;
    __ql__QLIterator_copy(&state->iter_a);
    __ql__QLIterator_copy(&state->iter_b);

    return iter;
}

static void* __ql__QLIterator_range_next(QLIterator* iter) {
    RangeIteratorState* state = (RangeIteratorState*)(iter->state);
    if ((state->step > 0 && state->current >= state->end) ||
        (state->step < 0 && state->current <= state->end)) {
        return NULL;
    }

    state->value = state->current;
    state->current += state->step;
    return &state->value;
}

static bool __ql__QLIterator_range_has_next(QLIterator* iter) {
    RangeIteratorState* state = (RangeIteratorState*)(iter->state);
    if (state->step > 0) {
        return state->current < state->end;
    }
    if (state->step < 0) {
        return state->current > state->end;
    }
    return false;
}

static void __ql__QLIterator_range_drop(QLIterator* iter) {
    free(iter);
}

QLIterator* __ql__QLIterator_range(int a, int b, int c) {
    if (c == 0) {
        fprintf(stderr, "range step cannot be 0\n");
        exit(1);
    }

    QLIterator* iter = __ql__QLIterator_new(
        __ql__QLIterator_range_next,
        __ql__QLIterator_range_has_next,
        __ql__QLIterator_range_drop,
        sizeof(RangeIteratorState),
        &__ql__int_type_info
    );

    RangeIteratorState* state = (RangeIteratorState*)(iter->state);
    state->current = a;
    state->end = b;
    state->step = c;
    state->value = 0;

    return iter;
}
