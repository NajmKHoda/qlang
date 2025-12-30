#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "../array.h"
#include "database.h"
#include "update_query.h"

UpdateQueryPlan* __ql__UpdateQueryPlan_new(char* table_name, QLTypeInfo* struct_type_info, unsigned int num_params) {
    UpdateQueryPlan* plan = malloc(sizeof(UpdateQueryPlan));
    plan->table_name = table_name;
    plan->struct_type_info = struct_type_info;
    plan->num_params = num_params;

    plan->num_assignments = 0;
    plan->assignments_capacity = 4;
    plan->assignments = malloc(sizeof(UpdateAssignment) * plan->assignments_capacity);

    plan->where.is_present = false;
    return plan;
}

void __ql__UpdateQueryPlan_add_assignment(
    UpdateQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
) {
    if (plan->num_assignments >= plan->assignments_capacity) {
        plan->assignments_capacity *= 2;
        plan->assignments = realloc(plan->assignments, sizeof(UpdateAssignment) * plan->assignments_capacity);
    }
    plan->assignments[plan->num_assignments].column_name = column_name;
    plan->assignments[plan->num_assignments].column_type = column_type;
    plan->assignments[plan->num_assignments].value = value;
    plan->num_assignments++;
}

void __ql__UpdateQueryPlan_set_where(
    UpdateQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
) {
    plan->where.is_present = true;
    plan->where.column_name = column_name;
    plan->where.column_type = column_type;
    plan->where.value = value;
}

PreparedQuery* __ql__UpdateQueryPlan_prepare(sqlite3* db, UpdateQueryPlan* plan) {
    char* sql = malloc(MAX_SQL_LENGTH);
    PreparedQuery* prepared_query = __ql__PreparedQuery_new(plan->num_params, NULL);
    
    // Build SET clause
    int next_index = 1;
    int offset = sprintf(sql, "UPDATE %s SET ", plan->table_name);
    for (unsigned int i = 0; i < plan->num_assignments; i++) {
        if (i > 0) offset += sprintf(sql + offset, ", ");

        int query_index;
        if (plan->assignments[i].column_type == QUERY_DATA_PARAMETER) {
            int param_index = *((int*)plan->assignments[i].value);
            unsigned int* param_query_index = &prepared_query->query_param_indices[param_index];
            if (*param_query_index == 0) {
                *param_query_index = next_index;
                query_index = next_index;
                next_index++;
            } else {
                query_index = *param_query_index;
            }
        } else {
            query_index = next_index;
            next_index++;
        }

        offset += sprintf(sql + offset, "%s = ?%d", plan->assignments[i].column_name, query_index);
    }
    
    // Add WHERE clause if present
    if (plan->where.is_present) {
        int where_index = next_index;
        if (plan->where.column_type == QUERY_DATA_PARAMETER) {
            int param_index = *((int*)plan->where.value);
            unsigned int* param_query_index = &prepared_query->query_param_indices[param_index];
            if (*param_query_index == 0) {
                *param_query_index = where_index;
            } else {
                where_index = *param_query_index;
            }
        }
        sprintf(sql + offset, " WHERE %s = ?%d;", plan->where.column_name, where_index);
    } else {
        sprintf(sql + offset, ";");
    }
    
    sqlite3_prepare_v2(db, sql, -1, &prepared_query->stmt, NULL);
    
    // Bind assignment values
    int query_index = 1;
    for (unsigned int i = 0; i < plan->num_assignments; i++) {
        switch (plan->assignments[i].column_type) {
            case QUERY_DATA_INTEGER: {
                sqlite3_bind_int(prepared_query->stmt, query_index, *((int*)plan->assignments[i].value));
                query_index++;
                break;
            }
            case QUERY_DATA_STRING: {
                QLString* ql_str = *(QLString**)plan->assignments[i].value;
                sqlite3_bind_text(prepared_query->stmt, query_index, ql_str->raw_string, ql_str->length, SQLITE_STATIC);
                query_index++;
                break;
            }
            case QUERY_DATA_PARAMETER: {
                int param_index = *((int*)plan->assignments[i].value);
                int param_query_index = prepared_query->query_param_indices[param_index];
                if (param_query_index == query_index) {
                    query_index++;
                }
                break;
            }
        }
    }
    
    // Bind WHERE value if present
    if (plan->where.is_present) {
        switch (plan->where.column_type) {
            case QUERY_DATA_INTEGER: {
                sqlite3_bind_int(prepared_query->stmt, next_index, *((int*)plan->where.value));
                break;
            }
            case QUERY_DATA_STRING: {
                QLString* ql_str = *(QLString**)plan->where.value;
                sqlite3_bind_text(prepared_query->stmt, next_index, ql_str->raw_string, ql_str->length, SQLITE_STATIC);
                break;
            }
            case QUERY_DATA_PARAMETER: {
                // Binding will occur later
                break;
            }
        }
    }

    free(sql);
    free(plan->assignments);
    free(plan);

    return prepared_query;
}
