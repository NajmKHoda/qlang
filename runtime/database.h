#ifndef RUNTIME_TABLE_H
#define RUNTIME_TABLE_H

#include <stdbool.h>

typedef struct sqlite3 sqlite3;

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

QLArray* __ql__PreparedQuery_execute(PreparedQuery* query);
void __ql__PreparedQuery_bind_scalar_param(PreparedQuery* query, unsigned int index, QueryDataType type, void* value);
void __ql__PreparedQuery_bind_row_param(PreparedQuery* query, unsigned int index, QLTypeInfo* struct_type_info, void* struct_ptr);
void __ql__PreparedQuery_add_ref(PreparedQuery* query);
void __ql__PreparedQuery_remove_ref(PreparedQuery* query);


typedef struct {
    bool is_present;
    char* column_name;
    QueryDataType column_type;
    void* value;
} WhereClause;

typedef struct {
    char* table_name;
    QLTypeInfo* struct_type_info;
    unsigned int num_params;
    WhereClause where;
} SelectQueryPlan;

SelectQueryPlan* __ql__SelectQueryPlan_new(char* table_name, QLTypeInfo* struct_type_info, unsigned int num_params);
void __ql__SelectQueryPlan_set_where(
    SelectQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
);
PreparedQuery* __ql__SelectQueryPlan_prepare(sqlite3* db, SelectQueryPlan* plan);


typedef struct {
    char* table_name;
    QLTypeInfo* struct_type_info;
    unsigned int num_params;
    bool is_parameter;
    void* data;
} InsertQueryPlan;

InsertQueryPlan* __ql__InsertQueryPlan_new(
    char* table_name,
    QLTypeInfo* struct_type_info,
    unsigned int num_params,
    bool is_parameter,
    void* data
);
PreparedQuery* __ql__InsertQueryPlan_prepare(sqlite3* db, InsertQueryPlan* plan);


typedef struct {
    char* table_name;
    unsigned int num_params;
    WhereClause where;
} DeleteQueryPlan;

DeleteQueryPlan* __ql__DeleteQueryPlan_new(char* table_name, unsigned int num_params);
void __ql__DeleteQueryPlan_set_where(
    DeleteQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
);
PreparedQuery* __ql__DeleteQueryPlan_prepare(sqlite3* db, DeleteQueryPlan* plan);


typedef struct {
    char* column_name;
    QueryDataType column_type;
    void* value;
} UpdateAssignment;

typedef struct {
    char* table_name;
    QLTypeInfo* struct_type_info;
    unsigned int num_params;

    UpdateAssignment* assignments;
    unsigned int num_assignments;
    unsigned int assignments_capacity;

    WhereClause where;
} UpdateQueryPlan;

UpdateQueryPlan* __ql__UpdateQueryPlan_new(char* table_name, QLTypeInfo* struct_type_info, unsigned int num_params);
void __ql__UpdateQueryPlan_add_assignment(
    UpdateQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
);
void __ql__UpdateQueryPlan_set_where(
    UpdateQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
);
PreparedQuery* __ql__UpdateQueryPlan_prepare(sqlite3* db, UpdateQueryPlan* plan);

#endif