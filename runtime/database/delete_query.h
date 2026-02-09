#ifndef RUNTIME_DELETE_QUERY_H
#define RUNTIME_DELETE_QUERY_H

#include <stdbool.h>

typedef struct {
    char* table_name;
    bool has_where_clause;
    char* where_column;
} DeletePlan;

typedef struct {
    sqlite3_stmt* stmt;
} PreparedDelete;

DeletePlan* __ql__DeletePlan_new(char* table_name);
void __ql__DeletePlan_set_where(DeletePlan* plan, char* column_name);
PreparedDelete* __ql__DeletePlan_prepare(sqlite3* db, DeletePlan* plan);

void __ql__PreparedDelete_bind_where(PreparedDelete* prepared_delete, QLType value_type, void* value);
void __ql__PreparedDelete_exec(PreparedDelete* prepared_delete);
void __ql__PreparedDelete_finalize(PreparedDelete* prepared_delete);

#endif
