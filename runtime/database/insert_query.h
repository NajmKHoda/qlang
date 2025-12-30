#ifndef RUNTIME_INSERT_QUERY_H
#define RUNTIME_INSERT_QUERY_H

#include <stdbool.h>
#include "database.h"

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

#endif
