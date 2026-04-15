#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "../array.h"
#include "database.h"
#include "insert_query.h"

InsertPlan* __ql__InsertPlan_new(char* table_name, QLTypeInfo* struct_type_info) {
    InsertPlan* plan = malloc(sizeof(InsertPlan));
    plan->table_name = table_name;
    plan->struct_type_info = struct_type_info;
    return plan;
}

PreparedInsert* __ql__InsertPlan_prepare(InsertPlan* plan) {
    PreparedInsert* prepared_insert = malloc(sizeof(PreparedInsert));
    prepared_insert->struct_type_info = plan->struct_type_info;

    char sql[MAX_SQL_LENGTH];
    char* writer = sql;
    writer += sprintf(writer, "INSERT INTO %s VALUES (?1", plan->table_name);

    unsigned int n_fields = plan->struct_type_info->num_fields;
    for (unsigned int i = 1; i < n_fields; i++) {
        writer += sprintf(writer, ", ?%d", i + 1);
    }
    writer += sprintf(writer, ");");

    if (sqlite3_prepare_v2(__ql__sqlite, sql, -1, &prepared_insert->stmt, NULL) != SQLITE_OK) {
        free(prepared_insert);
        free(plan);
        return NULL;
    }
    free(plan);
    return prepared_insert;
}

bool __ql__PreparedInsert_exec_row(PreparedInsert* prepared_insert, void* row) {
    if (prepared_insert == NULL) {
        return false;
    }

    unsigned int n_fields = prepared_insert->struct_type_info->num_fields;
    for (unsigned int i = 0; i < n_fields; i++) {
        ColumnType field_type;
        void* field_ptr = prepared_insert->struct_type_info->get_nth(row, i, &field_type);
        __ql__bind_value(prepared_insert->stmt, i + 1, field_type, field_ptr);
    }
    int step_rc = sqlite3_step(prepared_insert->stmt);
    int reset_rc = sqlite3_reset(prepared_insert->stmt);
    return step_rc == SQLITE_DONE && reset_rc == SQLITE_OK;
}

bool __ql__PreparedInsert_exec_array(PreparedInsert* prepared_insert, QLArray* array) {
    if (prepared_insert == NULL || array == NULL) {
        return false;
    }

    for (unsigned int i = 0; i < array->num_elems; i++) {
        void* elem_ptr = __ql__QLArray_index(array, i);
        if (!__ql__PreparedInsert_exec_row(prepared_insert, elem_ptr)) {
            return false;
        }
    }
    return true;
}

void __ql__PreparedInsert_finalize(PreparedInsert* prepared_insert) {
    if (prepared_insert == NULL) {
        return;
    }

    sqlite3_finalize(prepared_insert->stmt);
    free(prepared_insert);
    fprintf(stderr, "finalize PreparedInsert\n");
}