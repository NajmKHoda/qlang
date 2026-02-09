#ifndef RUNTIME_UPDATE_QUERY_H
#define RUNTIME_UPDATE_QUERY_H

#include <stdbool.h>
#include "../metadata.h"

typedef struct {
    char* table_name;
    unsigned int num_assignments;
    char** assign_columns;
    bool has_where_clause;
    char* where_column;
} UpdatePlan;

UpdatePlan* __ql__UpdatePlan_new(
    char* table_name,
    unsigned int num_assignments,
    char** assign_columns
);

typedef struct {
    sqlite3_stmt* stmt;
} PreparedUpdate;

void __ql__UpdatePlan_set_where(UpdatePlan* plan, char* column_name);
PreparedUpdate* __ql__UpdatePlan_prepare(sqlite3* db, UpdatePlan* plan);

void __ql__PreparedUpdate_bind_where(PreparedUpdate* prepared_update, QLType value_type, void* value);
void __ql__PreparedUpdate_bind_assignment(
    PreparedUpdate* prepared_update,
    unsigned int index,
    QLType value_type,
    void* value
);
void __ql__PreparedUpdate_exec(PreparedUpdate* prepared_update);
void __ql__PreparedUpdate_finalize(PreparedUpdate* prepared_update);

#endif
