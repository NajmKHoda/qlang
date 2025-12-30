#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "../array.h"
#include "database.h"
#include "select_query.h"

SelectQueryPlan* __ql__SelectQueryPlan_new(char* table_name, QLTypeInfo* struct_type_info, unsigned int num_params) {
    SelectQueryPlan* plan = malloc(sizeof(SelectQueryPlan));
    plan->table_name = table_name;
    plan->struct_type_info = struct_type_info;
    plan->num_params = num_params;
    plan->where.is_present = false;
    return plan;
}

void __ql__SelectQueryPlan_set_where(
    SelectQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
) {
    plan->where.is_present = true;
    plan->where.column_name = column_name;
    plan->where.column_type = column_type;
    plan->where.value = value;
}

PreparedQuery* __ql__SelectQueryPlan_prepare(sqlite3* db, SelectQueryPlan* plan) {
    PreparedQuery* prepared_query = __ql__PreparedQuery_new(plan->num_params, plan->struct_type_info);

    char* sql = malloc(MAX_SQL_LENGTH);

    if (plan->where.is_present) {
        sprintf(sql, "SELECT * FROM %s WHERE %s = ?1;", plan->table_name, plan->where.column_name);
        sqlite3_prepare_v2(db, sql, -1, &prepared_query->stmt, NULL);
        switch (plan->where.column_type) {
            case QUERY_DATA_INTEGER: {
                sqlite3_bind_int(prepared_query->stmt, 1, *((int*)plan->where.value));
                break;
            }
            case QUERY_DATA_STRING: {
                QLString* ql_str = *(QLString**)plan->where.value;
                sqlite3_bind_text(prepared_query->stmt, 1, ql_str->raw_string, ql_str->length, SQLITE_STATIC);
                break;
            }
            case QUERY_DATA_PARAMETER: {
                int param_index = *((int*)plan->where.value);
                prepared_query->query_param_indices[param_index] = 1;
                break;
            }
        }
    } else {
        sprintf(sql, "SELECT * FROM %s;", plan->table_name);
        sqlite3_prepare_v2(db, sql, -1, &prepared_query->stmt, NULL);
    }

    free(sql);
    free(plan);
    return prepared_query;
}
