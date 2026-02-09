#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "definitions.h"
#include "update_query.h"

UpdatePlan* __ql__UpdatePlan_new(
    char* table_name,
    unsigned int num_assignments,
    char** assign_columns
) {
    UpdatePlan* plan = malloc(sizeof(UpdatePlan));
    plan->table_name = table_name;
    plan->num_assignments = num_assignments;
    plan->assign_columns = assign_columns;
    plan->has_where_clause = false;
    return plan;
}

void __ql__UpdatePlan_set_where(UpdatePlan* plan, char* column_name) {
    plan->has_where_clause = true;
    plan->where_column = column_name;
}

PreparedUpdate* __ql__UpdatePlan_prepare(sqlite3* db, UpdatePlan* plan) {
    PreparedUpdate* prepared_update = malloc(sizeof(PreparedUpdate));

    char sql[MAX_SQL_LENGTH];
    char* writer = sql;

    // Build SET clause
    writer += sprintf(writer, "UPDATE %s SET ", plan->table_name);
    for (unsigned int i = 0; i < plan->num_assignments; i++) {
        if (i > 0) writer += sprintf(writer, ", ");
        writer += sprintf(writer, "%s = ?%d", plan->assign_columns[i], i + 2);
    }
    
    // Add WHERE clause if present
    if (plan->has_where_clause) {
        writer += sprintf(writer, " WHERE %s = ?1;", plan->where_column);
    } else {
        writer += sprintf(writer, ";");
    }
    
    sqlite3_prepare_v2(db, sql, -1, &prepared_update->stmt, NULL);
    free(plan);
    return prepared_update;
}

void __ql__PreparedUpdate_bind_where(PreparedUpdate* prepared_update, QLType value_type, void* value) {
    __ql__bind_value(prepared_update->stmt, 1, value_type, value);
}

void __ql__PreparedUpdate_bind_assignment(
    PreparedUpdate* prepared_update,
    unsigned int index,
    QLType value_type,
    void* value
) {
    __ql__bind_value(prepared_update->stmt, index + 2, value_type, value);
}

void __ql__PreparedUpdate_exec(PreparedUpdate* prepared_update) {
    sqlite3_step(prepared_update->stmt);
    sqlite3_reset(prepared_update->stmt);
}

void __ql__PreparedUpdate_finalize(PreparedUpdate* prepared_update) {
    sqlite3_finalize(prepared_update->stmt);
    free(prepared_update);
}
