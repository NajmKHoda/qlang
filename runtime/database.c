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

sqlite3* __ql__init_database(char* db_filename) {
    sqlite3* db;
    if (sqlite3_open(db_filename, &db) != SQLITE_OK) {
        fprintf(stderr, "Cannot open database: %s\n", sqlite3_errmsg(db));
        exit(1);
    }
    return db;
}

QueryPlan* __ql__QueryPlan_new(char* table_name, QLTypeInfo* struct_type_info) {
    QueryPlan* plan = malloc(sizeof(QueryPlan));
    plan->table_name = table_name;
    plan->struct_type_info = struct_type_info;
    plan->where.is_present = false;
    return plan;
}

void __ql__QueryPlan_set_where(
    QueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
) {
    plan->where.is_present = true;
    plan->where.column_name = column_name;
    plan->where.column_type = column_type;
    plan->where.value = value;
}

QLArray* __ql__QueryPlan_execute(sqlite3* db, QueryPlan* plan) {
    char* sql = malloc(MAX_SQL_LENGTH);
    sqlite3_stmt* stmt;
    if (plan->where.is_present) {
        sprintf(sql, "SELECT * FROM %s WHERE %s = ?;", plan->table_name, plan->where.column_name);
        sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);
        switch (plan->where.column_type) {
            case QUERY_INTEGER:
                sqlite3_bind_int(stmt, 1, *((int*)plan->where.value));
                break;
            case QUERY_STRING:
                QLString* ql_str = *(QLString**)plan->where.value;
                sqlite3_bind_text(stmt, 1, ql_str->raw_string, ql_str->length, SQLITE_STATIC);
                break;
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

    return results;
}