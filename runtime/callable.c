#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "qlstring.h"
#include "array.h"
#include "metadata.h"
#include "database/select_query.h"
#include "database/insert_query.h"
#include "database/update_query.h"
#include "database/delete_query.h"
#include "iterator.h"
#include "callable.h"

const QLTypeInfo __ql__QLCallable_type_info = {
    .size = sizeof(QLCallable*),
    .copy = (void (*)(void*)) __ql__QLCallable_copy,
    .drop = (void (*)(void*)) __ql__QLCallable_drop
};

QLCallable* __ql__QLCallable_new(void* true_invoke_fn, void* failable_invoke_fn, CallableType type, QLTypeInfo* captured_info) {
    QLCallable* callable = malloc(sizeof(QLCallable));
    callable->invoke_fn = true_invoke_fn;
    callable->failable_invoke_fn = failable_invoke_fn;
    callable->type = type;
    callable->context_info = captured_info;
    callable->prepared_stmt = NULL;
    callable->ref_count = 1;

    if (captured_info != NULL) {
        callable->context_struct = malloc(captured_info->size);
    } else {
        callable->context_struct = NULL;
    }

    return callable;
}

void __ql__QLCallable_set_stmt(QLCallable* callable, void* prepared_stmt) {
    callable->prepared_stmt = prepared_stmt;
}

void* __ql__QLCallable_get_fn(QLCallable* callable) {
    return callable->invoke_fn;
}

void* __ql__QLCallable_get_failable_fn(QLCallable* callable) {
    if (callable->failable_invoke_fn != NULL) {
        return callable->failable_invoke_fn;
    }
    return callable->invoke_fn;
}

void* __ql__QLCallable_get_context(QLCallable* callable) {
    return callable->context_struct;
}

void* __ql__QLCallable_get_stmt(QLCallable* callable) {
    return callable->prepared_stmt;
}

void __ql__QLCallable_copy(QLCallable** callable_ptr) {
    QLCallable* callable = *callable_ptr;
    callable->ref_count++;
}

void __ql__QLCallable_drop(QLCallable** callable_ptr) {
    QLCallable* callable = *callable_ptr;
    callable->ref_count--;
    if (callable->ref_count == 0) {
        fprintf(stderr, "free(callable %d)\n", callable->type);
        switch (callable->type) {
            case CALLABLE_SELECT: 
                __ql__QLIterator_drop((QLIterator**)&callable->prepared_stmt);
                break;
            case CALLABLE_INSERT: {
                PreparedInsert* prepared_insert = (PreparedInsert*)callable->prepared_stmt;
                __ql__PreparedInsert_finalize(prepared_insert);
                break;
            }
            case CALLABLE_UPDATE: {
                PreparedUpdate* prepared_update = (PreparedUpdate*)callable->prepared_stmt;
                __ql__PreparedUpdate_finalize(prepared_update);
                break;
            }
            case CALLABLE_DELETE: {
                PreparedDelete* prepared_delete = (PreparedDelete*)callable->prepared_stmt;
                __ql__PreparedDelete_finalize(prepared_delete);
                break;
            }
            default:
                break;
        }
        if (callable->context_struct != NULL) {
            if (callable->context_info->drop != NULL) {
                callable->context_info->drop(callable->context_struct);
            }
            free(callable->context_struct);
        }
        free(callable);
    }
}