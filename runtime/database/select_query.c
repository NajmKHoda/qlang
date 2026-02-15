#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "../array.h"
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

PreparedSelect* __ql__SelectPlan_prepare(sqlite3* db, SelectPlan* plan) {
    PreparedSelect* prepared_select = malloc(sizeof(PreparedSelect));
    prepared_select->struct_type_info = plan->struct_type_info;

    char sql[MAX_SQL_LENGTH];
    if (plan->has_where_clause) {
        sprintf(sql, "SELECT * FROM %s WHERE %s = ?1;", plan->table_name, plan->where_column);
        sqlite3_prepare_v2(db, sql, -1, &prepared_select->stmt, NULL);
    } else {
        sprintf(sql, "SELECT * FROM %s;", plan->table_name);
        sqlite3_prepare_v2(db, sql, -1, &prepared_select->stmt, NULL);
    }

    free(plan);
    return prepared_select;
}

void __ql__PreparedSelect_bind_where(PreparedSelect* prepared_select, QLType value_type, void* value) {
    __ql__bind_value(prepared_select->stmt, 1, value_type, value);
}

QLArray* __ql__PreparedSelect_execute(PreparedSelect* prepared_select) {
    QLArray* results = __ql__QLArray_new(NULL, 0, prepared_select->struct_type_info);

    int n_cols = prepared_select->struct_type_info->num_fields;
    void* struct_ptr = malloc(prepared_select->struct_type_info->size);
    while (sqlite3_step(prepared_select->stmt) == SQLITE_ROW) {
        for (int i = 0; i < n_cols; i++) {
            StructField field = prepared_select->struct_type_info->fields[i];
            void* field_ptr = (char*)struct_ptr + field.offset;
            int column_type = sqlite3_column_type(prepared_select->stmt, i);
            switch (column_type) {
                case SQLITE_TEXT: {
                    const unsigned char* text = sqlite3_column_text(prepared_select->stmt, i);
                    unsigned int length = sqlite3_column_bytes(prepared_select->stmt, i);
                    QLString* val = __ql__QLString_new(malloc(length), length, false);
                    memcpy(val->raw_string, text, length);
                    *(QLString**)field_ptr = val;
                    break;
                }
                case SQLITE_INTEGER: {
                    int val = sqlite3_column_int(prepared_select->stmt, i);
                    *(int*)field_ptr = val;
                    break;
                }
            }
            
        }
        __ql__QLArray_append(results, struct_ptr);
    }

    free(struct_ptr);
    sqlite3_reset(prepared_select->stmt);
    return results;
}

void __ql__PreparedSelect_finalize(PreparedSelect* prepared_select) {
    sqlite3_finalize(prepared_select->stmt);
    free(prepared_select);
    fprintf(stderr, "finalize PreparedSelect\n");
}
