#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "qlstring.h"
#include "array.h"
#include "memory.h"
#include "metadata.h"
#include "database/select_query.h"
#include "database/insert_query.h"
#include "database/update_query.h"
#include "database/delete_query.h"
#include "callable.h"

const QLTypeInfo __ql__QLCallable_type_info = {
    .type = TYPE_CALLABLE,
    .size = sizeof(QLCallable*)
};

QLCallable* __ql__QLCallable_new(void* invoke_fn, CallableType type, QLTypeInfo* captured_info) {
    QLCallable* callable = malloc(sizeof(QLCallable));
    callable->invoke_fn = invoke_fn;
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

void __ql__QLCallable_capture(QLCallable* callable, unsigned int index, void* value_ptr) {
    StructField field = callable->context_info->fields[index];
    void* field_ptr = (char*)callable->context_struct + field.offset;
    memcpy(field_ptr, value_ptr, field.type_info->size);
}

void* __ql__QLCallable_get_fn(QLCallable* callable) {
    return callable->invoke_fn;
}

void* __ql__QLCallable_get_context(QLCallable* callable) {
    return callable->context_struct;
}

void* __ql__QLCallable_get_stmt(QLCallable* callable) {
    return callable->prepared_stmt;
}

void __ql__QLCallable_add_ref(QLCallable* callable) {
    callable->ref_count++;
}

void __ql__QLCallable_remove_ref(QLCallable* callable) {
    callable->ref_count--;
    if (callable->ref_count == 0) {
        if (callable->context_info != NULL) {
            __ql__drop_value(callable->context_struct, callable->context_info);
        }

        switch (callable->type) {
            case CALLABLE_SELECT: {
                PreparedSelect* prepared_select = (PreparedSelect*)callable->prepared_stmt;
                __ql__PreparedSelect_finalize(prepared_select);
                break;
            }
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
            free(callable->context_struct);
        }
        free(callable);
        fprintf(stderr, "free(callable %d)\n", callable->type);
        
    }
}