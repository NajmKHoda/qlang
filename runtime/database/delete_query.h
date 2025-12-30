#ifndef RUNTIME_DELETE_QUERY_H
#define RUNTIME_DELETE_QUERY_H

#include <stdbool.h>
#include "database.h"

typedef struct {
    bool is_present;
    char* column_name;
    QueryDataType column_type;
    void* value;
} DeleteWhereClause;

typedef struct {
    char* table_name;
    unsigned int num_params;
    DeleteWhereClause where;
} DeleteQueryPlan;

DeleteQueryPlan* __ql__DeleteQueryPlan_new(char* table_name, unsigned int num_params);
void __ql__DeleteQueryPlan_set_where(
    DeleteQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
);
PreparedQuery* __ql__DeleteQueryPlan_prepare(sqlite3* db, DeleteQueryPlan* plan);

#endif
