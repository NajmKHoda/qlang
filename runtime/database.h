#ifndef RUNTIME_TABLE_H
#define RUNTIME_TABLE_H

#include <stdbool.h>

typedef struct sqlite3 sqlite3;

void __ql__init_dbs_from_args(int argc, char** argv, int num_dbs, sqlite3*** db_globals);
void __ql__close_dbs(int num_dbs, sqlite3*** db_globals);

typedef enum {
    QUERY_DATA_INTEGER,
    QUERY_DATA_STRING
} QueryDataType;

typedef struct {
    bool is_present;
    char* column_name;
    QueryDataType column_type;
    void* value;
} WhereClause;


typedef struct {
    char* table_name;
    QLTypeInfo* struct_type_info;
    WhereClause where;
} SelectQueryPlan;

SelectQueryPlan* __ql__SelectQueryPlan_new(char* table_name, QLTypeInfo* struct_type_info);
void __ql__SelectQueryPlan_set_where(
    SelectQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
);
QLArray* __ql__SelectQueryPlan_execute(sqlite3* db, SelectQueryPlan* plan);


typedef struct {
    char* table_name;
    QLTypeInfo* struct_type_info;
    bool is_single_row;
    void* data;
} InsertQueryPlan;

InsertQueryPlan* __ql__InsertQueryPlan_new(
    char* table_name,
    QLTypeInfo* struct_type_info,
    bool is_single_row,
    void* data
);
void __ql__InsertQueryPlan_execute(sqlite3* db, InsertQueryPlan* plan);


typedef struct {
    char* table_name;
    WhereClause where;
} DeleteQueryPlan;

DeleteQueryPlan* __ql__DeleteQueryPlan_new(char* table_name);
void __ql__DeleteQueryPlan_set_where(
    DeleteQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
);
void __ql__DeleteQueryPlan_execute(sqlite3* db, DeleteQueryPlan* plan);

#endif