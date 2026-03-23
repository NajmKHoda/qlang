#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "../iterator.h"
#include "definitions.h"
#include "select_query.h"

static void __ql__PreparedSelect_copy(PreparedSelect** prepared_select_ptr);
static void __ql__PreparedSelect_drop(PreparedSelect** prepared_select_ptr);

QLTypeInfo __ql__PreparedSelect_type_info = {
    .size = sizeof(PreparedSelect*),
    .copy = (void (*)(void*)) __ql__PreparedSelect_copy,
    .drop = (void (*)(void*)) __ql__PreparedSelect_drop
};

static void* __ql__PreparedSelect_iter_next(QLIterator* iter) {
    PreparedSelect* prepared_select = (PreparedSelect*)iter->iterable;
    switch (prepared_select->state) {
        case PREPARED_SELECT_STATE_READY:
            prepared_select->state = PREPARED_SELECT_STATE_NEXT;
            break;
        case PREPARED_SELECT_STATE_NEXT:
            if (sqlite3_step(prepared_select->stmt) != SQLITE_ROW) {
                fprintf(stderr, "next() called on exhausted iterator\n");
                exit(1);
            }
            break;
        case PREPARED_SELECT_STATE_EXHAUSTED:
            fprintf(stderr, "next() called on exhausted iterator\n");
            exit(1);
    }
    
    int n_cols = prepared_select->struct_type_info->num_fields;
    for (int i = 0; i < n_cols; i++) {
        int column_type = sqlite3_column_type(prepared_select->stmt, i);
        switch (column_type) {
            case SQLITE_TEXT: {
                const unsigned char* text = sqlite3_column_text(prepared_select->stmt, i);
                unsigned int length = sqlite3_column_bytes(prepared_select->stmt, i);
                QLString* val = __ql__QLString_new(malloc(length), length, false);
                memcpy(val->raw_string, text, length);
                prepared_select->struct_type_info->set_nth(prepared_select->row_ptr, i, &val);
                break;
            }
            case SQLITE_INTEGER: {
                int val = sqlite3_column_int(prepared_select->stmt, i);
                prepared_select->struct_type_info->set_nth(prepared_select->row_ptr, i, &val);
                break;
            }
        }
    }

    return prepared_select->row_ptr;
}

static bool __ql__PreparedSelect_iter_has_next(QLIterator* iter) {
    PreparedSelect* prepared_select = (PreparedSelect*)iter->iterable;
    switch (prepared_select->state) {
        case PREPARED_SELECT_STATE_NEXT:
            if (sqlite3_step(prepared_select->stmt) == SQLITE_ROW) {
                prepared_select->state = PREPARED_SELECT_STATE_READY;
                return true;
            } else {
                prepared_select->state = PREPARED_SELECT_STATE_EXHAUSTED;
                return false;
            }
        case PREPARED_SELECT_STATE_READY:
            return true;
        case PREPARED_SELECT_STATE_EXHAUSTED:
            return false;
    }
    return false;
}

SelectPlan* __ql__SelectPlan_new(char* table_name, QLTypeInfo* struct_type_info) {
    SelectPlan* plan = malloc(sizeof(SelectPlan));
    plan->table_name = table_name;
    plan->struct_type_info = struct_type_info;
    plan->has_where_clause = false;
    return plan;
}

void __ql__SelectPlan_set_where(SelectPlan* plan, char* column_name){
    plan->has_where_clause = true;
    plan->where_column = column_name;
}

PreparedSelect* __ql__SelectPlan_prepare(sqlite3* db, SelectPlan* plan) {
    PreparedSelect* prepared_select = malloc(sizeof(PreparedSelect));
    prepared_select->struct_type_info = plan->struct_type_info;
    prepared_select->row_ptr = malloc(plan->struct_type_info->size);
    prepared_select->state = PREPARED_SELECT_STATE_NEXT;
    prepared_select->ref_count = 1;

    char* sql = malloc(MAX_SQL_LENGTH);
    if (plan->has_where_clause) {
        sprintf(sql, "SELECT * FROM %s WHERE %s = ?1;", plan->table_name, plan->where_column);
        sqlite3_prepare_v2(db, sql, -1, &prepared_select->stmt, NULL);
    } else {
        sprintf(sql, "SELECT * FROM %s;", plan->table_name);
        sqlite3_prepare_v2(db, sql, -1, &prepared_select->stmt, NULL);
    }
    prepared_select->sql = sql;

    free(plan);
    return prepared_select;
}

PreparedSelect* __ql__PreparedSelect_copy_if_needed(PreparedSelect* prepared_select) {
    if (prepared_select->ref_count > 1) {
        if (prepared_select->sql == NULL) {
            fprintf(stderr, "Cannot copy a copy of a PreparedSelect. This is a logical compiler error.");
            exit(1);
        }

        // This prepared statement is in use; create a new copy
        PreparedSelect* new_copy = malloc(sizeof(PreparedSelect));
        sqlite3* db = sqlite3_db_handle(prepared_select->stmt);
        new_copy->sql = NULL; // SQL only in original
        sqlite3_prepare_v2(db, prepared_select->sql, -1, &new_copy->stmt, NULL);
        new_copy->struct_type_info = prepared_select->struct_type_info;
        new_copy->row_ptr = malloc(prepared_select->struct_type_info->size);
        new_copy->state = PREPARED_SELECT_STATE_NEXT;
        new_copy->ref_count = 0; // Caller will set to 1 after getting the copy

        return new_copy;
    } else {
        return prepared_select;
    }
}

void __ql__PreparedSelect_bind_where(PreparedSelect* prepared_select, ColumnType value_type, void* value) {
    __ql__bind_value(prepared_select->stmt, 1, value_type, value);
}

QLIterator* __ql__PreparedSelect_execute(PreparedSelect* prepared_select) {
    QLIterator* iter = __ql__QLIterator_new(
        prepared_select,
        __ql__PreparedSelect_iter_next,
        __ql__PreparedSelect_iter_has_next,
        &__ql__PreparedSelect_type_info,
        prepared_select->struct_type_info
    );

    return iter;
}

void __ql__PreparedSelect_finalize(PreparedSelect* prepared_select) {
    prepared_select->ref_count--;
    if (prepared_select->ref_count > 0) {
        return;
    }

    sqlite3_finalize(prepared_select->stmt);
    free(prepared_select->row_ptr);
    if (prepared_select->sql != NULL) {
        free(prepared_select->sql);
    }
    free(prepared_select);
    fprintf(stderr, "finalize PreparedSelect\n");
}

static void __ql__PreparedSelect_copy(PreparedSelect** prepared_select_ptr) {
    PreparedSelect* prepared_select = *prepared_select_ptr;
    prepared_select->ref_count++;
}

static void __ql__PreparedSelect_drop(PreparedSelect** prepared_select_ptr) {
    PreparedSelect* prepared_select = *prepared_select_ptr;
    __ql__PreparedSelect_finalize(prepared_select);
}
