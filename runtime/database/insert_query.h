#ifndef RUNTIME_INSERT_QUERY_H
#define RUNTIME_INSERT_QUERY_H

#include <stdbool.h>
#include "database.h"

typedef struct {
    QLTypeInfo* struct_type_info;
    char* table_name;
} InsertPlan;

typedef struct {
    sqlite3_stmt* stmt;
    QLTypeInfo* struct_type_info;
} PreparedInsert;

InsertPlan* __ql__InsertPlan_new(char* table_name, QLTypeInfo* struct_type_info);
PreparedInsert* __ql__InsertPlan_prepare(sqlite3* db, InsertPlan* plan);

void __ql__PreparedInsert_exec_row(PreparedInsert* prepared_insert, void* row);
void __ql__PreparedInsert_exec_array(PreparedInsert* prepared_insert, QLArray* array);
void __ql__PreparedInsert_finalize(PreparedInsert* prepared_insert);

#endif
