#ifndef RUNTIME_SELECT_QUERY_H
#define RUNTIME_SELECT_QUERY_H

#include <stdbool.h>
#include "../metadata.h"

typedef struct {
    QLTypeInfo* struct_type_info;
    char* table_name;
    bool has_where_clause;
    char* where_column;
} SelectPlan;

typedef struct {
    sqlite3_stmt* stmt;
    QLTypeInfo* struct_type_info;
} PreparedSelect;

SelectPlan* __ql__SelectPlan_new(char* table_name, QLTypeInfo* struct_type_info);
void __ql__SelectPlan_set_where(SelectPlan* plan, char* column_name);
PreparedSelect* __ql__SelectPlan_prepare(sqlite3* db, SelectPlan* plan);

void __ql__PreparedSelect_bind_where(PreparedSelect* prepared_select, QLType value_type, void* value);
QLArray* __ql__PreparedSelect_execute(PreparedSelect* prepared_select);
void __ql__PreparedSelect_finalize(PreparedSelect* prepared_select);

#endif