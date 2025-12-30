#ifndef RUNTIME_UPDATE_QUERY_H
#define RUNTIME_UPDATE_QUERY_H

#include <stdbool.h>
#include "database.h"

typedef struct {
    bool is_present;
    char* column_name;
    QueryDataType column_type;
    void* value;
} UpdateWhereClause;

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

    UpdateWhereClause where;
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
