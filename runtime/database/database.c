#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "../qlstring.h"
#include "../array.h"
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

PreparedQuery* __ql__PreparedQuery_new(unsigned int num_params, QLTypeInfo* return_type_info) {
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
