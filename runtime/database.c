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

// --- SELECT ---

SelectQueryPlan* __ql__SelectQueryPlan_new(char* table_name, QLTypeInfo* struct_type_info) {
    SelectQueryPlan* plan = malloc(sizeof(SelectQueryPlan));
    plan->table_name = table_name;
    plan->struct_type_info = struct_type_info;
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

QLArray* __ql__SelectQueryPlan_execute(sqlite3* db, SelectQueryPlan* plan) {
    char* sql = malloc(MAX_SQL_LENGTH);
    sqlite3_stmt* stmt;
    if (plan->where.is_present) {
        sprintf(sql, "SELECT * FROM %s WHERE %s = ?;", plan->table_name, plan->where.column_name);
        sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);
        switch (plan->where.column_type) {
            case QUERY_DATA_INTEGER: {
                sqlite3_bind_int(stmt, 1, *((int*)plan->where.value));
                break;
            }
            case QUERY_DATA_STRING: {
                QLString* ql_str = *(QLString**)plan->where.value;
                sqlite3_bind_text(stmt, 1, ql_str->raw_string, ql_str->length, SQLITE_STATIC);
                break;
            }
        }
    } else {
        sprintf(sql, "SELECT * FROM %s;", plan->table_name);
        sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);
    }

    QLArray* results = __ql__QLArray_new(NULL, 0, plan->struct_type_info);

    int ncols = sqlite3_column_count(stmt);
    void* temp_struct = malloc(plan->struct_type_info->size);
    while (sqlite3_step(stmt) == SQLITE_ROW) {
        for (int i = 0; i < ncols; i++) {
            switch (sqlite3_column_type(stmt, i)) {
                case SQLITE_INTEGER: {
                    int val = sqlite3_column_int(stmt, i);
                    plan->struct_type_info->set_nth(temp_struct, i, &val);
                    break;
                }
                case SQLITE_TEXT: {
                    const unsigned char* text = sqlite3_column_text(stmt, i);
                    int length = sqlite3_column_bytes(stmt, i);
                    char* str_buf = malloc(length);
                    memcpy(str_buf, text, length);
                    QLString* ql_str = __ql__QLString_new(str_buf, length, false);
                    plan->struct_type_info->set_nth(temp_struct, i, &ql_str);
                    break;
                }
            }
        }
        __ql__QLArray_append(results, temp_struct);
    }
    free(temp_struct);

    sqlite3_finalize(stmt);
    free(sql);
    free(plan);

    return results;
}

// --- INSERT ---

InsertQueryPlan* __ql__InsertQueryPlan_new(
    char* table_name,
    QLTypeInfo* struct_type_info,
    bool is_single_row,
    void* data
) {
    InsertQueryPlan* plan = malloc(sizeof(InsertQueryPlan));
    plan->table_name = table_name;
    plan->struct_type_info = struct_type_info;
    plan->is_single_row = is_single_row;
    plan->data = data;
    return plan;
}

void bind_row(sqlite3_stmt* stmt, QLTypeInfo* struct_type_info, void* struct_ptr) {
    unsigned int num_columns = struct_type_info->num_columns;
    for (unsigned int i = 0; i < num_columns; i++) {
        int datatype;
        void* column_ptr;
        struct_type_info->get_nth(struct_ptr, i, &datatype, &column_ptr);
        switch (datatype) {
            case QUERY_DATA_INTEGER: {
                int val = *(int*)column_ptr;
                sqlite3_bind_int(stmt, i + 1, val);
                break;
            }
            case QUERY_DATA_STRING: {
                QLString* ql_str = *(QLString**)column_ptr;
                sqlite3_bind_text(stmt, i + 1, ql_str->raw_string, ql_str->length, SQLITE_STATIC);
                break;
            }
        }
    }
}

void __ql__InsertQueryPlan_execute(sqlite3* db, InsertQueryPlan* plan) {
    char* sql = malloc(MAX_SQL_LENGTH);
    sqlite3_stmt* stmt;

    unsigned int num_columns = plan->struct_type_info->num_columns;
    unsigned int placeholders_size = num_columns * 2;
    char* placeholders = malloc(placeholders_size);
    placeholders[0] = '?';
    placeholders[placeholders_size - 1] = '\0';
    for (int i = 1; i < num_columns; i++) {
        placeholders[i * 2 - 1] = ',';
        placeholders[i * 2] = '?';
    }

    sprintf(sql, "INSERT INTO %s VALUES (%s);", plan->table_name, placeholders);
    free(placeholders);

    sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);

    if (plan->is_single_row) {
        void* struct_ptr = plan->data;
        bind_row(stmt, plan->struct_type_info, struct_ptr);
        sqlite3_step(stmt);
    } else {
        QLArray* array = *(QLArray**)plan->data;
        for (unsigned int j = 0; j < array->num_elems; j++) {
            void* struct_ptr = __ql__QLArray_index(array, j);
            bind_row(stmt, plan->struct_type_info, struct_ptr);
            sqlite3_step(stmt);
            sqlite3_reset(stmt);
        }
    }

    sqlite3_finalize(stmt);
    free(sql);
    free(plan);
}

// --- DELETE ---

DeleteQueryPlan* __ql__DeleteQueryPlan_new(char* table_name) {
    DeleteQueryPlan* plan = malloc(sizeof(DeleteQueryPlan));
    plan->table_name = table_name;
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

void __ql__DeleteQueryPlan_execute(sqlite3* db, DeleteQueryPlan* plan) {
    char* sql = malloc(MAX_SQL_LENGTH);
    sqlite3_stmt* stmt;
    
    if (plan->where.is_present) {
        sprintf(sql, "DELETE FROM %s WHERE %s = ?;", plan->table_name, plan->where.column_name);
        sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);
        switch (plan->where.column_type) {
            case QUERY_DATA_INTEGER: {
                sqlite3_bind_int(stmt, 1, *((int*)plan->where.value));
                break;
            }
            case QUERY_DATA_STRING: {
                QLString* ql_str = *(QLString**)plan->where.value;
                sqlite3_bind_text(stmt, 1, ql_str->raw_string, ql_str->length, SQLITE_STATIC);
                break;
            }
        }
    } else {
        sprintf(sql, "DELETE FROM %s;", plan->table_name);
        sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);
    }
    
    sqlite3_step(stmt);
    
    sqlite3_finalize(stmt);
    free(sql);
    free(plan);
}
