#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "../array.h"
#include "database.h"
#include "delete_query.h"

DeleteQueryPlan* __ql__DeleteQueryPlan_new(char* table_name, unsigned int num_params) {
    DeleteQueryPlan* plan = malloc(sizeof(DeleteQueryPlan));
    plan->table_name = table_name;
    plan->num_params = num_params;
    plan->where.is_present = false;
    return plan;
}

void __ql__DeleteQueryPlan_set_where(
    DeleteQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
) {
    plan->where.is_present = true;
    plan->where.column_name = column_name;
    plan->where.column_type = column_type;
    plan->where.value = value;
}

PreparedQuery* __ql__DeleteQueryPlan_prepare(sqlite3* db, DeleteQueryPlan* plan) {
    char* sql = malloc(MAX_SQL_LENGTH);
    PreparedQuery* prepared_query = __ql__PreparedQuery_new(plan->num_params, NULL);
    
    if (plan->where.is_present) {
        sprintf(sql, "DELETE FROM %s WHERE %s = ?;", plan->table_name, plan->where.column_name);
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
        sprintf(sql, "DELETE FROM %s;", plan->table_name);
        sqlite3_prepare_v2(db, sql, -1, &prepared_query->stmt, NULL);
    }

    free(sql);
    free(plan);

    return prepared_query;
}
