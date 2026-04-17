#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <sqlite3.h>
#include "../metadata.h"
#include "database.h"
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

PreparedUpdate* __ql__UpdatePlan_prepare(UpdatePlan* plan) {
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
    
    if (sqlite3_prepare_v2(__ql__sqlite, sql, -1, &prepared_update->stmt, NULL) != SQLITE_OK) {
        free(prepared_update);
        free(plan);
        return NULL;
    }
    free(plan);
    return prepared_update;
}

bool __ql__PreparedUpdate_bind_where(PreparedUpdate* prepared_update, ColumnType value_type, void* value) {
    if (prepared_update == NULL) {
        return false;
    }

    __ql__bind_value(prepared_update->stmt, 1, value_type, value);
    return sqlite3_errcode(__ql__sqlite) == SQLITE_OK;
}

bool __ql__PreparedUpdate_bind_assignment(
    PreparedUpdate* prepared_update,
    unsigned int index,
    ColumnType value_type,
    void* value
) {
    if (prepared_update == NULL) {
        return false;
    }

    __ql__bind_value(prepared_update->stmt, index + 2, value_type, value);
    return sqlite3_errcode(__ql__sqlite) == SQLITE_OK;
}

bool __ql__PreparedUpdate_exec(PreparedUpdate* prepared_update) {
    if (prepared_update == NULL) {
        return false;
    }

    int step_rc = sqlite3_step(prepared_update->stmt);
    int reset_rc = sqlite3_reset(prepared_update->stmt);
    return step_rc == SQLITE_DONE && reset_rc == SQLITE_OK;
}

void __ql__PreparedUpdate_finalize(PreparedUpdate* prepared_update) {
    if (prepared_update == NULL) {
        return;
    }

    sqlite3_finalize(prepared_update->stmt);
    free(prepared_update);
}
