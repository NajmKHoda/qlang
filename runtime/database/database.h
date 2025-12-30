#ifndef RUNTIME_DATABASE_H
#define RUNTIME_DATABASE_H

#include <stdbool.h>

typedef struct sqlite3 sqlite3;
typedef struct sqlite3_stmt sqlite3_stmt;

void __ql__init_dbs_from_args(int argc, char** argv, int num_dbs, sqlite3*** db_globals);
void __ql__close_dbs(int num_dbs, sqlite3*** db_globals);

typedef enum {
    QUERY_DATA_INTEGER,
    QUERY_DATA_STRING,
    QUERY_DATA_PARAMETER
} QueryDataType;

typedef struct {
    sqlite3_stmt* stmt;
    unsigned int num_params;
    unsigned int* query_param_indices;
    QLTypeInfo* return_type_info;
    unsigned int ref_count;
} PreparedQuery;

extern const unsigned int MAX_SQL_LENGTH;

PreparedQuery* __ql__PreparedQuery_new(unsigned int num_params, QLTypeInfo* return_type_info);
QLArray* __ql__PreparedQuery_execute(PreparedQuery* query);
void __ql__PreparedQuery_bind_scalar_param(PreparedQuery* query, unsigned int index, QueryDataType type, void* value);
void __ql__PreparedQuery_bind_row_param(PreparedQuery* query, unsigned int index, QLTypeInfo* struct_type_info, void* struct_ptr);
void __ql__PreparedQuery_add_ref(PreparedQuery* query);
void __ql__PreparedQuery_remove_ref(PreparedQuery* query);

#endif
