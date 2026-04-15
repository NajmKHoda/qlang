#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "../iterator.h"
#include "database.h"
#include "select_query.h"

SelectPlan* __ql__SelectPlan_new(
    char* table_name,
    unsigned int num_columns,
    unsigned int num_joins,
    QLTypeInfo* struct_type_info
) {
    SelectPlan* plan = malloc(sizeof(SelectPlan));
    plan->table_name = table_name;
    plan->struct_type_info = struct_type_info;
    plan->num_columns = num_columns;
    plan->columns = malloc(num_columns * sizeof(QColumn));
    plan->num_joins = num_joins;
    plan->join_clauses = malloc(num_joins * sizeof(JoinClause));
    plan->has_where_clause = false;
    plan->has_limit_clause = false;
    plan->has_offset_clause = false;
    return plan;
}

void __ql__SelectPlan_set_column(SelectPlan* plan, unsigned int index, unsigned int table_index, char* column_name) {
    QColumn* col = &plan->columns[index];
    col->table_index = table_index;
    col->column_name = column_name;
}

void __ql__SelectPlan_set_join(
    SelectPlan* plan,
    unsigned int index,
    char* right_table_name,
    unsigned int left_table_index,
    char* left_column_name,
    unsigned int right_table_index,
    char* right_column_name
) {
    JoinClause* join_clause = &plan->join_clauses[index];
    join_clause->right_table_name = right_table_name;
    join_clause->left_column.table_index = left_table_index;
    join_clause->left_column.column_name = left_column_name;
    join_clause->right_column.table_index = right_table_index;
    join_clause->right_column.column_name = right_column_name;
}

void __ql__SelectPlan_set_where(SelectPlan* plan, unsigned int table_index, char* column_name){
    plan->has_where_clause = true;
    plan->where_column.table_index = table_index;
    plan->where_column.column_name = column_name;
}

void __ql__SelectPlan_set_limit(SelectPlan* plan) {
    plan->has_limit_clause = true;
}

void __ql__SelectPlan_set_offset(SelectPlan* plan) {
    plan->has_offset_clause = true;
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
}

QLIterator* __ql__SelectPlan_prepare(SelectPlan* plan) {
    QLIterator* select_iterator = __ql__QLIterator_new(
        __ql__SelectIterator_next,
        __ql__SelectIterator_has_next,
        __ql__SelectIterator_drop,
        sizeof(SelectIteratorState),
        plan->struct_type_info
    );

    SelectIteratorState* state = (SelectIteratorState*)select_iterator->state;
    state->row_ptr = malloc(plan->struct_type_info->size);
    state->where_bind_index = 0;
    state->limit_bind_index = 0;
    state->offset_bind_index = 0;
    state->state = SELECT_ITERATOR_EXHAUSTED;

    char* sql = malloc(MAX_SQL_LENGTH);
    char* write_ptr = sql;

    write_ptr += sprintf(write_ptr, "SELECT ");
    for (unsigned int i = 0; i < plan->num_columns; i++) {
        QColumn col = plan->columns[i];
        write_ptr += sprintf(write_ptr, "t%u.%s", col.table_index, col.column_name);
        if (i < plan->num_columns - 1) {
            write_ptr += sprintf(write_ptr, ", ");
        }
    }
    write_ptr += sprintf(write_ptr, " FROM %s AS t0", plan->table_name);

    for (unsigned int i = 0; i < plan->num_joins; i++) {
        JoinClause join = plan->join_clauses[i];
        write_ptr += sprintf(write_ptr, " JOIN %s AS t%u ON t%u.%s = t%u.%s",
            join.right_table_name,
            join.right_column.table_index,
            join.left_column.table_index, join.left_column.column_name,
            join.right_column.table_index, join.right_column.column_name
        );
    }

    unsigned int next_bind_index = 1;
    if (plan->has_where_clause) {
        state->where_bind_index = next_bind_index;
        write_ptr += sprintf(
            write_ptr,
            " WHERE t%u.%s = ?%u",
            plan->where_column.table_index,
            plan->where_column.column_name,
            state->where_bind_index
        );
        next_bind_index++;
    }

    if (plan->has_limit_clause) {
        state->limit_bind_index = next_bind_index;
        write_ptr += sprintf(write_ptr, " LIMIT ?%u", state->limit_bind_index);
        next_bind_index++;
    }

    if (plan->has_offset_clause) {
        state->offset_bind_index = next_bind_index;
        write_ptr += sprintf(write_ptr, " OFFSET ?%u", state->offset_bind_index);
    }
    write_ptr += sprintf(write_ptr, ";");

    /*
    if (plan->has_where_clause) {
        sprintf(sql, "SELECT * FROM %s WHERE %s = ?1;", plan->table_name, plan->where_column.column_name);
        sqlite3_prepare_v2(db, sql, -1, &state->stmt, NULL);
    } else {
        sprintf(sql, "SELECT * FROM %s;", plan->table_name);
        sqlite3_prepare_v2(db, sql, -1, &state->stmt, NULL);
    }
    */
    if (sqlite3_prepare_v2(__ql__sqlite, sql, -1, &state->stmt, NULL) != SQLITE_OK) {
        free(state->row_ptr);
        free(sql);
        free(select_iterator);
        free(plan);
        return NULL;
    }
    state->sql = sql;

    free(plan);
    return select_iterator;
}


QLIterator* __ql__SelectIterator_activate(QLIterator* select_iterator) {
    if (select_iterator == NULL) {
        return NULL;
    }

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

        sqlite3_prepare_v2(__ql__sqlite, state->sql, -1, &new_state->stmt, NULL);
        if (new_state->stmt == NULL) {
            free(new_iterator);
            return NULL;
        }
        new_state->row_ptr = malloc(select_iterator->elem_type_info->size);
        new_state->where_bind_index = state->where_bind_index;
        new_state->limit_bind_index = state->limit_bind_index;
        new_state->offset_bind_index = state->offset_bind_index;
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

bool __ql__SelectIterator_bind_where(QLIterator* select_iterator, ColumnType value_type, void* value) {
    if (select_iterator == NULL) {
        return false;
    }

    SelectIteratorState* state = (SelectIteratorState*)select_iterator->state;
    if (state->where_bind_index > 0) {
        __ql__bind_value(state->stmt, state->where_bind_index, value_type, value);
    }
    return sqlite3_errcode(__ql__sqlite) == SQLITE_OK;
}

bool __ql__SelectIterator_bind_limit(QLIterator* select_iterator, void* value) {
    if (select_iterator == NULL) {
        return false;
    }

    SelectIteratorState* state = (SelectIteratorState*)select_iterator->state;
    if (state->limit_bind_index > 0) {
        int raw = *(int*)value;
        return sqlite3_bind_int(state->stmt, state->limit_bind_index, raw < 0 ? 0 : raw) == SQLITE_OK;
    }
    return true;
}

bool __ql__SelectIterator_bind_offset(QLIterator* select_iterator, void* value) {
    if (select_iterator == NULL) {
        return false;
    }

    SelectIteratorState* state = (SelectIteratorState*)select_iterator->state;
    if (state->offset_bind_index > 0) {
        int raw = *(int*)value;
        return sqlite3_bind_int(state->stmt, state->offset_bind_index, raw < 0 ? 0 : raw) == SQLITE_OK;
    }
    return true;
}
