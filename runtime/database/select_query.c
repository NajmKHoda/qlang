#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "../iterator.h"
#include "definitions.h"
#include "select_query.h"

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

static void* __ql__SelectIterator_next(QLIterator* iter) {
    SelectIteratorState* state = (SelectIteratorState*)iter->state;
    switch (state->state) {
        case SELECT_ITERATOR_READY:
            state->state = SELECT_ITERATOR_NEXT;
            break;
        case SELECT_ITERATOR_NEXT:
            if (sqlite3_step(state->stmt) != SQLITE_ROW) {
                return NULL;
            }
            break;
        case SELECT_ITERATOR_EXHAUSTED:
            return NULL;
    }
    
    int n_cols = iter->elem_type_info->num_fields;
    for (int i = 0; i < n_cols; i++) {
        int column_type = sqlite3_column_type(state->stmt, i);
        switch (column_type) {
            case SQLITE_TEXT: {
                const unsigned char* text = sqlite3_column_text(state->stmt, i);
                unsigned int length = sqlite3_column_bytes(state->stmt, i);
                QLString* val = __ql__QLString_new(malloc(length), length, false);
                memcpy(val->raw_string, text, length);
                iter->elem_type_info->set_nth(state->row_ptr, i, &val);
                break;
            }
            case SQLITE_INTEGER: {
                int val = sqlite3_column_int(state->stmt, i);
                iter->elem_type_info->set_nth(state->row_ptr, i, &val);
                break;
            }
        }
    }

    return state->row_ptr;
}

static bool __ql__SelectIterator_has_next(QLIterator* iter) {
    SelectIteratorState* state = (SelectIteratorState*)iter->state;
    switch (state->state) {
        case SELECT_ITERATOR_NEXT:
            if (sqlite3_step(state->stmt) == SQLITE_ROW) {
                state->state = SELECT_ITERATOR_READY;
                return true;
            } else {
                state->state = SELECT_ITERATOR_EXHAUSTED;
                return false;
            }
        case SELECT_ITERATOR_READY:
            return true;
        case SELECT_ITERATOR_EXHAUSTED:
            return false;
    }
    return false;
}

static void __ql__SelectIterator_drop(QLIterator* iter) {
    SelectIteratorState* state = (SelectIteratorState*)iter->state;
    sqlite3_finalize(state->stmt);
    free(state->row_ptr);
    if (state->sql != NULL) {
        free(state->sql);
    }
    free(iter);
    fprintf(stderr, "finalize SelectIterator\n");
}

QLIterator* __ql__SelectPlan_prepare(sqlite3* db, SelectPlan* plan) {
    QLIterator* select_iterator = __ql__QLIterator_new(
        __ql__SelectIterator_next,
        __ql__SelectIterator_has_next,
        __ql__SelectIterator_drop,
        sizeof(SelectIteratorState),
        plan->struct_type_info
    );

    SelectIteratorState* state = (SelectIteratorState*)select_iterator->state;
    state->row_ptr = malloc(plan->struct_type_info->size);
    state->state = SELECT_ITERATOR_EXHAUSTED;

    char* sql = malloc(MAX_SQL_LENGTH);
    if (plan->has_where_clause) {
        sprintf(sql, "SELECT * FROM %s WHERE %s = ?1;", plan->table_name, plan->where_column);
        sqlite3_prepare_v2(db, sql, -1, &state->stmt, NULL);
    } else {
        sprintf(sql, "SELECT * FROM %s;", plan->table_name);
        sqlite3_prepare_v2(db, sql, -1, &state->stmt, NULL);
    }
    state->sql = sql;

    free(plan);
    return select_iterator;
}


QLIterator* __ql__SelectIterator_activate(QLIterator* select_iterator) {
    SelectIteratorState* state = (SelectIteratorState*)select_iterator->state;
    if (state->sql == NULL) {
        fprintf(stderr, "Cannot activate a SelectIterator clone. This is a logical compiler error.");
        exit(1);
    }

    if (select_iterator->ref_count > 1) {
        // This prepared statement is in use; create a new copy
        QLIterator* new_iterator = __ql__QLIterator_new(
            __ql__SelectIterator_next,
            __ql__SelectIterator_has_next,
            __ql__SelectIterator_drop,
            sizeof(SelectIteratorState),
            select_iterator->elem_type_info
        );
        SelectIteratorState* new_state = (SelectIteratorState*)new_iterator->state;

        sqlite3* db = sqlite3_db_handle(state->stmt);
        sqlite3_prepare_v2(db, state->sql, -1, &new_state->stmt, NULL);
        new_state->row_ptr = malloc(select_iterator->elem_type_info->size);
        new_state->state = SELECT_ITERATOR_NEXT;
        new_state->sql = NULL; // SQL only in original
        return new_iterator;
    } else {
        // This prepared statement is not in use; we can reuse it
        sqlite3_reset(state->stmt);
        sqlite3_clear_bindings(state->stmt);
        state->state = SELECT_ITERATOR_NEXT;
        select_iterator->ref_count++;
        return select_iterator;
    }
}

void __ql__SelectIterator_bind_where(QLIterator* select_iterator, ColumnType value_type, void* value) {
    SelectIteratorState* state = (SelectIteratorState*)select_iterator->state;
    __ql__bind_value(state->stmt, 1, value_type, value);
}
