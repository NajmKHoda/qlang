#ifndef RUNTIME_SELECT_QUERY_H
#define RUNTIME_SELECT_QUERY_H

#include <stdbool.h>
#include "database.h"

typedef struct {
    bool is_present;
    char* column_name;
    QueryDataType column_type;
    void* value;
} SelectWhereClause;

typedef struct {
    char* table_name;
    QLTypeInfo* struct_type_info;
    unsigned int num_params;
    SelectWhereClause where;
} SelectQueryPlan;

SelectQueryPlan* __ql__SelectQueryPlan_new(char* table_name, QLTypeInfo* struct_type_info, unsigned int num_params);
void __ql__SelectQueryPlan_set_where(
    SelectQueryPlan* plan,
    char* column_name,
    QueryDataType column_type,
    void* value
);
PreparedQuery* __ql__SelectQueryPlan_prepare(sqlite3* db, SelectQueryPlan* plan);

#endif
