#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "../array.h"
#include "database.h"
#include "insert_query.h"

static void bind_row(sqlite3_stmt* stmt, unsigned int query_index, QLTypeInfo* struct_type_info, void* struct_ptr) {
    unsigned int num_columns = struct_type_info->num_columns;
    for (unsigned int i = 0; i < num_columns; i++) {
        int datatype;
        void* column_ptr;
        struct_type_info->get_nth(struct_ptr, i, &datatype, &column_ptr);
        switch (datatype) {
            case QUERY_DATA_INTEGER: {
                int val = *(int*)column_ptr;
                sqlite3_bind_int(stmt, query_index + i, val);
                break;
            }
            case QUERY_DATA_STRING: {
                QLString* ql_str = *(QLString**)column_ptr;
                sqlite3_bind_text(stmt, query_index + i, ql_str->raw_string, ql_str->length, SQLITE_STATIC);
                break;
            }
        }
    }
}

InsertQueryPlan* __ql__InsertQueryPlan_new(
    char* table_name,
    QLTypeInfo* struct_type_info,
    unsigned int num_params,
    bool is_parameter,
    void* data
) {
    InsertQueryPlan* plan = malloc(sizeof(InsertQueryPlan));
    plan->table_name = table_name;
    plan->struct_type_info = struct_type_info;
    plan->num_params = num_params;
    plan->is_parameter = is_parameter;
    plan->data = data;
    return plan;
}

PreparedQuery* __ql__InsertQueryPlan_prepare(sqlite3* db, InsertQueryPlan* plan) {
    char* sql = malloc(MAX_SQL_LENGTH);
    PreparedQuery* prepared_query = __ql__PreparedQuery_new(plan->num_params, NULL);

    unsigned int num_columns = plan->struct_type_info->num_columns;
    int offset = sprintf(sql, "INSERT INTO %s VALUES (?1", plan->table_name);
    for (int i = 1; i < num_columns; i++) {
        offset += sprintf(sql + offset, ", ?%d", i + 1);
    }
    sprintf(sql + offset, ");");

    sqlite3_prepare_v2(db, sql, -1, &prepared_query->stmt, NULL);

    if (plan->is_parameter) {
        int param_index = *((int*)plan->data);
        prepared_query->query_param_indices[param_index] = 1;
    } else {
        bind_row(prepared_query->stmt, 1, plan->struct_type_info, plan->data);
    }

    free(sql);
    free(plan);

    return prepared_query;
}
