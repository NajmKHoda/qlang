#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "metadata.h"
#include "qlstring.h"
#include "array.h"
#include "database.h"

const unsigned int MAX_SQL_LENGTH = 512;

void __ql__init_dbs_from_args(int argc, char** argv, int num_dbs, sqlite3*** db_globals) {
    argc--; argv++;
    if (argc < num_dbs) {
        fprintf(stderr, "Expected %d database file paths, got %d\n", num_dbs, argc);
        exit(1);
    }
    for (int i = 0; i < argc; i++) {
        sqlite3* db;
        if (sqlite3_open(argv[i], &db) != SQLITE_OK) {
            fprintf(stderr, "Cannot open database: %s\n", sqlite3_errmsg(db));
            sqlite3_close(db);
            for (int j = 0; j < i; j++) {
                sqlite3_close(*(db_globals[j]));
            }
            exit(1);
        }
        *(db_globals[i]) = db;
    }
}

void __ql__close_dbs(int num_dbs, sqlite3*** db_globals) {
    for (int i = 0; i < num_dbs; i++) {
        sqlite3_close(*(db_globals[i]));
    }
}


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


static PreparedQuery* __ql__PreparedQuery_new(unsigned int num_params, QLTypeInfo* return_type_info) {
    PreparedQuery* query = malloc(sizeof(PreparedQuery));
    query->stmt = NULL;
    query->num_params = num_params;
    query->query_param_indices = num_params > 0 ? calloc(num_params, sizeof(unsigned int)) : NULL;
    query->ref_count = 1;
    query->return_type_info = return_type_info;
    return query;
}

void __ql__PreparedQuery_bind_scalar_param(PreparedQuery* query, unsigned int index, QueryDataType type, void* value) {
    unsigned int query_index = query->query_param_indices[index];
    switch (type) {
        case QUERY_DATA_INTEGER: {
            int val = *((int*)value);
            sqlite3_bind_int(query->stmt, query_index, val);
            break;
        }
        case QUERY_DATA_STRING: {
            QLString* ql_str = *(QLString**)value;
            sqlite3_bind_text(query->stmt, query_index, ql_str->raw_string, ql_str->length, SQLITE_STATIC);
            break;
        }
        case QUERY_DATA_PARAMETER: {
            fprintf(stderr, "Compiler logic error: unexpected QUERY_DATA_PARAMETER in bind_scalar_param\n");
            exit(1);
            break;
        }
    }
}

void __ql__PreparedQuery_bind_row_param(PreparedQuery* query, unsigned int index, QLTypeInfo* struct_type_info, void* struct_ptr) {
    unsigned int query_index = query->query_param_indices[index];
    bind_row(query->stmt, query_index, struct_type_info, struct_ptr);
}

QLArray* __ql__PreparedQuery_execute(PreparedQuery* query) {
    if (query->return_type_info == NULL) {
        sqlite3_step(query->stmt);
        sqlite3_reset(query->stmt);
        return NULL;
    }

    QLArray* results = __ql__QLArray_new(NULL, 0, query->return_type_info);
    int ncols = sqlite3_column_count(query->stmt);
    void* temp_struct = malloc(query->return_type_info->size);
    while (sqlite3_step(query->stmt) == SQLITE_ROW) {
        for (int i = 0; i < ncols; i++) {
            switch (sqlite3_column_type(query->stmt, i)) {
                case SQLITE_INTEGER: {
                    int val = sqlite3_column_int(query->stmt, i);
                    query->return_type_info->set_nth(temp_struct, i, &val);
                    break;
                }
                case SQLITE_TEXT: {
                    const unsigned char* text = sqlite3_column_text(query->stmt, i);
                    int length = sqlite3_column_bytes(query->stmt, i);
                    char* str_buf = malloc(length);
                    memcpy(str_buf, text, length);
                    QLString* ql_str = __ql__QLString_new(str_buf, length, false);
                    query->return_type_info->set_nth(temp_struct, i, &ql_str);
                    break;
                }
            }
        }
        __ql__QLArray_append(results, temp_struct);
    }

    free(temp_struct);
    sqlite3_reset(query->stmt);

    return results;
}

void __ql__PreparedQuery_add_ref(PreparedQuery* query) {
    query->ref_count++;
}

void __ql__PreparedQuery_remove_ref(PreparedQuery* query) {
    query->ref_count--;
    if (query->ref_count == 0) {
        sqlite3_finalize(query->stmt);
        if (query->query_param_indices != NULL) {
            free(query->query_param_indices);
        }
        free(query);
    }
}

// --- SELECT ---

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

// --- INSERT ---

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

// --- DELETE ---

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

// --- UPDATE ---

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
